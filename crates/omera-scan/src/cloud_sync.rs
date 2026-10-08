//! Stable cloud namespace planning. No remote requests or source mutations.

use omera_domain::{CloudSyncNamespaceManifest, CloudSyncOptions, CloudSyncRootMapping};
use omera_storage::Database;
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};

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

#[cfg(test)]
mod tests {
    use super::*;

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
