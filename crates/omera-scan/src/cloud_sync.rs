//! Stable cloud namespace planning. No remote requests or source mutations.

use omera_domain::{
    CloudSyncNamespaceManifest, CloudSyncNamespacePreview, CloudSyncOptions, CloudSyncRootMapping,
};
use omera_storage::Database;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct SyncItem {
    pub local_path: PathBuf,
    pub remote_key: String,
    pub size_bytes: u64,
    pub mtime_secs: i64,
}

pub struct CloudSyncPlan {
    pub manifest: CloudSyncNamespaceManifest,
    pub items: Vec<SyncItem>,
    pub total_bytes: u64,
}

fn basename(path: &str) -> String {
    path.replace('\\', "/")
        .trim_end_matches('/')
        .rsplit('/')
        .next()
        .filter(|part| !part.is_empty())
        .unwrap_or("library")
        .to_string()
}

fn validate_object_path(path: &str) -> Result<(), String> {
    for part in path.split('/') {
        let reserved = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        if part.is_empty()
            || matches!(part, "." | "..")
            || part.ends_with(['.', ' '])
            || part
                .chars()
                .any(|ch| ch.is_control() || "\\:%?#*<>|\"".contains(ch))
            || matches!(reserved.as_str(), "CON" | "PRN" | "AUX" | "NUL")
            || (reserved.len() == 4
                && (reserved.starts_with("COM") || reserved.starts_with("LPT"))
                && matches!(reserved.as_bytes()[3], b'1'..=b'9'))
        {
            return Err(format!(
                "Unsafe or nonportable cloud path component: {part}"
            ));
        }
    }
    Ok(())
}

fn under_prefix(prefix: &str, suffix: &str) -> String {
    if prefix.is_empty() {
        suffix.to_string()
    } else {
        format!("{prefix}/{suffix}")
    }
}

pub fn build_namespace_manifest(
    db: &Database,
    options: &CloudSyncOptions,
) -> Result<CloudSyncNamespaceManifest, String> {
    let prefix = options.remote_prefix.trim_matches('/');
    if !prefix.is_empty() {
        validate_object_path(prefix)?;
    }
    let mut folders = db.list_folders().map_err(|e| e.to_string())?;
    folders.sort_by_key(|folder| folder.id);
    let selected = options.folder_ids.as_ref().filter(|ids| !ids.is_empty());
    if let Some(ids) = selected {
        if ids
            .iter()
            .any(|id| !folders.iter().any(|folder| folder.id == *id))
        {
            return Err("A selected library folder no longer exists".into());
        }
    }
    let mut legacy_counts = HashMap::<String, usize>::new();
    for folder in &folders {
        let name = db
            .cloud_sync_root_identity(folder.id)
            .map_err(|e| e.to_string())?
            .map(|identity| identity.legacy_basename)
            .unwrap_or_else(|| basename(&folder.path));
        *legacy_counts.entry(name.to_lowercase()).or_default() += 1;
    }
    let mut roots = Vec::new();
    let mut namespaces = HashSet::new();
    for folder in folders {
        if selected.is_some_and(|ids| !ids.contains(&folder.id)) {
            continue;
        }
        let identity = db
            .ensure_cloud_sync_root(folder.id)
            .map_err(|e| e.to_string())?;
        // A corrupted identity must never become an arbitrary remote path.
        let uuid = &identity.root_uuid;
        if uuid.len() != 36
            || !uuid.bytes().enumerate().all(|(i, ch)| {
                if matches!(i, 8 | 13 | 18 | 23) {
                    ch == b'-'
                } else {
                    ch.is_ascii_hexdigit()
                }
            })
        {
            return Err("Invalid persisted cloud root UUID".into());
        }
        let new_prefix = under_prefix(prefix, &format!("v2/roots/{uuid}"));
        if !namespaces.insert(new_prefix.to_lowercase()) {
            return Err("Cloud root namespaces collide".into());
        }
        roots.push(CloudSyncRootMapping {
            folder_id: folder.id,
            source_path: folder.path,
            root_uuid: uuid.clone(),
            legacy_prefix: under_prefix(prefix, &identity.legacy_basename),
            new_prefix,
            legacy_ambiguous: legacy_counts[&identity.legacy_basename.to_lowercase()] > 1,
        });
    }
    Ok(CloudSyncNamespaceManifest {
        version: 2,
        remote_prefix: prefix.to_string(),
        roots,
    })
}

pub fn namespace_manifest_id(manifest: &CloudSyncNamespaceManifest) -> Result<String, String> {
    let bytes = serde_json::to_vec(manifest).map_err(|e| e.to_string())?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

/// Persist a content-addressed compatibility manifest without replacing earlier plans.
/// Call outside the shared database guard and before starting any remote transfer.
pub fn save_namespace_manifest(
    data_dir: &Path,
    manifest: CloudSyncNamespaceManifest,
) -> Result<CloudSyncNamespacePreview, String> {
    let manifest_id = namespace_manifest_id(&manifest)?;
    let bytes = serde_json::to_vec(&manifest).map_err(|e| e.to_string())?;
    let dir = data_dir.join("cloud-sync-manifests");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Cannot save cloud layout manifest: {e}"))?;
    let path = dir.join(format!("layout-{manifest_id}.json"));
    let mut staged = tempfile::NamedTempFile::new_in(&dir).map_err(|e| e.to_string())?;
    staged.write_all(&bytes).map_err(|e| e.to_string())?;
    staged.as_file().sync_all().map_err(|e| e.to_string())?;
    match staged.persist_noclobber(&path) {
        Ok(_) => {}
        Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
            if std::fs::read(&path).map_err(|e| e.to_string())? != bytes {
                return Err("Existing cloud layout manifest is damaged; it was preserved".into());
            }
        }
        Err(error) => {
            return Err(format!(
                "Cannot publish cloud layout manifest: {}",
                error.error
            ))
        }
    }
    Ok(CloudSyncNamespacePreview {
        manifest_id,
        manifest_path: path.to_string_lossy().into_owned(),
        manifest,
    })
}

/// Preflight the complete selection before handing any item to a transfer worker.
pub fn collect_sync_plan(
    db: &Database,
    options: &CloudSyncOptions,
) -> Result<CloudSyncPlan, String> {
    let manifest = build_namespace_manifest(db, options)?;
    let manifest_id = namespace_manifest_id(&manifest)?;
    if options.namespace_manifest_id.as_deref() != Some(&manifest_id) {
        return Err("Preview and confirm the current cloud folder layout before syncing".into());
    }
    let mut items = Vec::new();
    let mut keys = HashSet::new();
    let mut total_bytes = 0u64;
    for root in &manifest.roots {
        let normalized_root = root.source_path.replace('\\', "/");
        let boundary = format!("{}/", normalized_root.trim_end_matches('/'));
        let files = db
            .list_file_fingerprints(root.folder_id)
            .map_err(|e| e.to_string())?;
        for (file_path, size_bytes, mtime_secs, _) in files {
            let normalized_file = file_path.replace('\\', "/");
            let relative = normalized_file
                .strip_prefix(&boundary)
                .ok_or_else(|| format!("Indexed file is outside its cloud root: {file_path}"))?;
            validate_object_path(relative)?;
            let remote_key = format!("{}/{relative}", root.new_prefix);
            // Conservative comparison also protects case-insensitive LocalPath targets.
            if !keys.insert(remote_key.to_lowercase()) {
                return Err(format!("Cloud object keys collide: {remote_key}"));
            }
            total_bytes = total_bytes
                .checked_add(size_bytes)
                .ok_or("Cloud sync byte total overflow")?;
            items.push(SyncItem {
                local_path: PathBuf::from(file_path),
                remote_key,
                size_bytes,
                mtime_secs,
            });
        }
    }
    Ok(CloudSyncPlan {
        manifest,
        items,
        total_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persisted_manifest_is_repeatable_and_never_overwrites_previous_evidence() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::connect_in_memory().unwrap();
        db.add_folder("/a/outputs").unwrap();
        let manifest = build_namespace_manifest(&db, &CloudSyncOptions::default()).unwrap();
        let saved = save_namespace_manifest(dir.path(), manifest.clone()).unwrap();
        let before = std::fs::read(&saved.manifest_path).unwrap();
        let decoded: CloudSyncNamespaceManifest = serde_json::from_slice(&before).unwrap();
        assert_eq!(decoded, manifest);
        let repeated = save_namespace_manifest(dir.path(), manifest).unwrap();
        assert_eq!(repeated.manifest_path, saved.manifest_path);
        let changed = build_namespace_manifest(
            &db,
            &CloudSyncOptions {
                remote_prefix: "new-target".into(),
                ..Default::default()
            },
        )
        .unwrap();
        let second = save_namespace_manifest(dir.path(), changed).unwrap();
        assert_ne!(second.manifest_path, saved.manifest_path);
        assert_eq!(std::fs::read(&saved.manifest_path).unwrap(), before);
        std::fs::write(&saved.manifest_path, b"existing damaged evidence").unwrap();
        assert!(save_namespace_manifest(dir.path(), saved.manifest).is_err());
        assert_eq!(
            std::fs::read(&saved.manifest_path).unwrap(),
            b"existing damaged evidence"
        );
        let blocker = dir.path().join("blocked-data-dir");
        std::fs::write(&blocker, b"unrelated file").unwrap();
        assert!(save_namespace_manifest(&blocker, second.manifest).is_err());
        assert_eq!(std::fs::read(blocker).unwrap(), b"unrelated file");
    }

    fn index(db: &Database, folder_id: i64, path: &str) {
        db.upsert_file(&omera_domain::ImageFile {
            id: None,
            folder_id,
            path: path.into(),
            size_bytes: 4,
            modified_at: 1,
            container: omera_domain::Container::Png,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        })
        .unwrap();
    }

    fn acknowledge(db: &Database, options: &mut CloudSyncOptions) {
        options.namespace_manifest_id =
            Some(namespace_manifest_id(&build_namespace_manifest(db, options).unwrap()).unwrap());
    }

    #[test]
    fn plan_requires_current_acknowledgement_and_isolates_duplicate_relative_names() {
        let db = Database::connect_in_memory().unwrap();
        let a = db.add_folder("/a/outputs").unwrap();
        let b = db.add_folder("/b/outputs").unwrap();
        index(&db, a.id, "/a/outputs/image.png");
        index(&db, b.id, "/b/outputs/image.png");
        let mut options = CloudSyncOptions::default();
        assert!(collect_sync_plan(&db, &options).is_err());
        acknowledge(&db, &mut options);
        let plan = collect_sync_plan(&db, &options).unwrap();
        assert_eq!(plan.total_bytes, 8);
        assert_eq!(plan.items.len(), 2);
        assert_ne!(plan.items[0].remote_key, plan.items[1].remote_key);
        assert!(plan
            .items
            .iter()
            .all(|item| item.remote_key.ends_with("/image.png")));
        options.remote_prefix = "changed".into();
        assert!(collect_sync_plan(&db, &options).is_err());
    }

    #[test]
    fn plan_rejects_case_collisions_and_root_escape_before_transfer() {
        for paths in [
            vec!["/a/outputs/A.png", "/a/outputs/a.png"],
            vec!["/a/outputs-extra/image.png"],
            vec!["/a/outputs/../secret.png"],
            vec!["/a/outputs/%2e%2e/image.png"],
        ] {
            let db = Database::connect_in_memory().unwrap();
            let folder = db.add_folder("/a/outputs").unwrap();
            for path in paths {
                index(&db, folder.id, path);
            }
            let mut options = CloudSyncOptions::default();
            acknowledge(&db, &mut options);
            assert!(collect_sync_plan(&db, &options).is_err());
        }
    }

    #[test]
    fn corrupt_or_case_colliding_root_identities_are_rejected() {
        let db = Database::connect_in_memory().unwrap();
        let a = db.add_folder("/a/outputs").unwrap();
        let b = db.add_folder("/b/outputs").unwrap();
        db.ensure_cloud_sync_root(a.id).unwrap();
        db.ensure_cloud_sync_root(b.id).unwrap();
        db.connection().execute(
            "UPDATE cloud_sync_roots SET root_uuid = 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa' WHERE folder_id = ?1", [a.id],
        ).unwrap();
        db.connection().execute(
            "UPDATE cloud_sync_roots SET root_uuid = 'AAAAAAAA-AAAA-AAAA-AAAA-AAAAAAAAAAAA' WHERE folder_id = ?1", [b.id],
        ).unwrap();
        assert!(build_namespace_manifest(&db, &CloudSyncOptions::default()).is_err());
        db.connection()
            .execute(
                "UPDATE cloud_sync_roots SET root_uuid = '../unsafe' WHERE folder_id = ?1",
                [b.id],
            )
            .unwrap();
        assert!(build_namespace_manifest(&db, &CloudSyncOptions::default()).is_err());
    }

    #[test]
    fn manifest_is_stable_and_flags_unselected_legacy_ambiguity() {
        let db = Database::connect_in_memory().unwrap();
        let a = db.add_folder("/a/outputs").unwrap();
        let b = db.add_folder("/b/outputs").unwrap();
        let options = CloudSyncOptions::default();
        let manifest = build_namespace_manifest(&db, &options).unwrap();
        assert_eq!(manifest.roots.len(), 2);
        assert!(manifest.roots.iter().all(|root| root.legacy_ambiguous));
        assert_ne!(manifest.roots[0].new_prefix, manifest.roots[1].new_prefix);
        assert_eq!(manifest, build_namespace_manifest(&db, &options).unwrap());
        let subset = build_namespace_manifest(
            &db,
            &CloudSyncOptions {
                folder_ids: Some(vec![a.id]),
                ..options.clone()
            },
        )
        .unwrap();
        assert!(subset.roots[0].legacy_ambiguous);
        assert!(db.cloud_sync_root_identity(b.id).unwrap().is_some());
        assert_ne!(
            namespace_manifest_id(&manifest).unwrap(),
            namespace_manifest_id(&subset).unwrap()
        );
        let empty_filter = CloudSyncOptions {
            folder_ids: Some(vec![]),
            ..options
        };
        assert_eq!(
            manifest,
            build_namespace_manifest(&db, &empty_filter).unwrap()
        );
    }

    #[test]
    fn unsafe_prefixes_and_unknown_roots_fail_before_mapping_allocation() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/a/outputs").unwrap();
        for prefix in ["../outside", "C:/absolute", "a//b", "a/%2e%2e", "a/CON"] {
            let options = CloudSyncOptions {
                remote_prefix: prefix.into(),
                ..Default::default()
            };
            assert!(build_namespace_manifest(&db, &options).is_err(), "{prefix}");
        }
        assert!(build_namespace_manifest(
            &db,
            &CloudSyncOptions {
                folder_ids: Some(vec![99]),
                ..Default::default()
            }
        )
        .is_err());
        assert_eq!(db.cloud_sync_root_identity(folder.id).unwrap(), None);
    }
}
