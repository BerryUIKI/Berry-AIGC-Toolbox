use super::*;
use omera_domain::{Container, ImageFile};
use omera_scan::cloud_sync::{
    build_namespace_manifest, collect_sync_plan, save_namespace_manifest, CloudSyncPlan,
};
use omera_storage::Database;

struct Fixture {
    dir: tempfile::TempDir,
    db: Database,
    options: CloudSyncOptions,
    plan: CloudSyncPlan,
}

impl Fixture {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::connect(&dir.path().join("omera.db")).unwrap();
        for (parent, content) in [("a", b"AAAA"), ("b", b"BBBB")] {
            let root = dir.path().join(parent).join("outputs");
            fs::create_dir_all(&root).unwrap();
            let path = root.join("image.png");
            fs::write(&path, content).unwrap();
            let folder = db.add_folder(root.to_str().unwrap()).unwrap();
            db.upsert_file(&ImageFile {
                id: None,
                folder_id: folder.id,
                path: path.to_string_lossy().into_owned(),
                size_bytes: 4,
                modified_at: 1,
                container: Container::Png,
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
        let mut options = CloudSyncOptions::default();
        let manifest = build_namespace_manifest(&db, &options).unwrap();
        assert!(manifest.roots.iter().all(|root| root.legacy_ambiguous));
        let preview = save_namespace_manifest(dir.path(), manifest).unwrap();
        options.namespace_manifest_id = Some(preview.manifest_id);
        let plan = collect_sync_plan(&db, &options).unwrap();
        assert_eq!(plan.items.len(), 2);
        assert_ne!(plan.items[0].remote_key, plan.items[1].remote_key);
        Self {
            dir,
            db,
            options,
            plan,
        }
    }

    fn assert_sources_unchanged(&self) {
        assert_eq!(self.db.get_database_stats().unwrap().file_count, 2);
        for (item, content) in self.plan.items.iter().zip([b"AAAA", b"BBBB"]) {
            assert_eq!(fs::read(&item.local_path).unwrap(), content);
        }
    }
}

fn limiter() -> Arc<Mutex<RateLimiter>> {
    Arc::new(Mutex::new(RateLimiter::new(0)))
}

#[test]
fn local_uploads_separate_same_basename_roots_and_preserve_legacy() {
    let fixture = Fixture::new();
    let remote = fixture.dir.path().join("remote");
    let legacy = remote.join("media/outputs/image.png");
    fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    fs::write(&legacy, b"legacy source").unwrap();
    let config = CloudBackupConfig {
        provider: CloudStorageProvider::LocalPath,
        local_path: Some(remote.to_string_lossy().into_owned()),
        ..Default::default()
    };
    for (item, content) in fixture.plan.items.iter().zip([b"AAAA", b"BBBB"]) {
        assert!(matches!(
            sync_single_item(&config, &fixture.options, item, &limiter()).unwrap(),
            SyncOutcome::Uploaded(4)
        ));
        assert_eq!(fs::read(remote.join(&item.remote_key)).unwrap(), content);
    }
    assert_eq!(fs::read(legacy).unwrap(), b"legacy source");
    fixture.assert_sources_unchanged();
}

#[test]
fn s3_uploads_use_distinct_uuid_keys_and_exact_source_bodies() {
    let fixture = Fixture::new();
    let mut server = mockito::Server::new();
    let config = CloudBackupConfig {
        provider: CloudStorageProvider::S3,
        s3_endpoint: Some(server.url()),
        s3_bucket: Some("test-bucket".into()),
        s3_region: Some("us-east-1".into()),
        s3_access_key: Some("fixture-access".into()),
        s3_secret_key: Some("fixture-secret".into()),
        s3_prefix: Some("backup".into()),
        ..Default::default()
    };
    let no_legacy = server
        .mock("PUT", "/test-bucket/backup/media/outputs/image.png")
        .expect(0)
        .create();
    let no_delete = server
        .mock("DELETE", mockito::Matcher::Any)
        .expect(0)
        .create();
    let options = CloudSyncOptions {
        strategy: CloudSyncStrategy::Sha256Checksum,
        ..fixture.options.clone()
    };
    for (item, content) in fixture.plan.items.iter().zip([b"AAAA", b"BBBB"]) {
        let path = format!("/test-bucket/backup/{}", item.remote_key);
        let head = server.mock("HEAD", path.as_str()).with_status(404).create();
        let put = server
            .mock("PUT", path.as_str())
            .match_body(content.to_vec())
            .match_header("x-amz-meta-sha256", sha256_hex(content).as_str())
            .with_status(200)
            .create();
        assert!(matches!(
            sync_single_item(&config, &options, item, &limiter()).unwrap(),
            SyncOutcome::Uploaded(4)
        ));
        head.assert();
        put.assert();
    }
    no_legacy.assert();
    no_delete.assert();
    fixture.assert_sources_unchanged();
}

#[test]
fn webdav_uploads_use_distinct_uuid_keys_and_exact_source_bodies() {
    let fixture = Fixture::new();
    let mut server = mockito::Server::new();
    let config = CloudBackupConfig {
        provider: CloudStorageProvider::WebDav,
        webdav_endpoint: Some(format!("{}/backup", server.url())),
        ..Default::default()
    };
    let no_legacy = server
        .mock("PUT", "/backup/media/outputs/image.png")
        .expect(0)
        .create();
    let no_delete = server
        .mock("DELETE", mockito::Matcher::Any)
        .expect(0)
        .create();
    let mkdir = server
        .mock("MKCOL", mockito::Matcher::Any)
        .with_status(201)
        .expect(4)
        .create();
    for (item, content) in fixture.plan.items.iter().zip([b"AAAA", b"BBBB"]) {
        let path = format!("/backup/{}", item.remote_key);
        let head = server.mock("HEAD", path.as_str()).with_status(404).create();
        let put = server
            .mock("PUT", path.as_str())
            .match_body(content.to_vec())
            .with_status(201)
            .create();
        assert!(matches!(
            sync_single_item(&config, &fixture.options, item, &limiter()).unwrap(),
            SyncOutcome::Uploaded(4)
        ));
        head.assert();
        put.assert();
    }
    mkdir.assert();
    no_legacy.assert();
    no_delete.assert();
    fixture.assert_sources_unchanged();
}

#[test]
fn renamed_root_keeps_remote_keys_after_restart_and_staged_backup_restore() {
    let fixture = Fixture::new();
    let Fixture {
        dir,
        db,
        mut options,
        plan,
    } = fixture;
    let original = &plan.manifest.roots[0];
    let old_root = PathBuf::from(&original.source_path);
    let new_root = dir.path().join("renamed");
    assert!(old_root.starts_with(dir.path()) && new_root.starts_with(dir.path()));
    fs::rename(&old_root, &new_root).unwrap();
    db.connection()
        .execute(
            "UPDATE folders SET path = ?1 WHERE id = ?2",
            (new_root.to_str().unwrap(), original.folder_id),
        )
        .unwrap();
    db.connection()
        .execute(
            "UPDATE files SET path = ?1 WHERE folder_id = ?2",
            (
                new_root.join("image.png").to_str().unwrap(),
                original.folder_id,
            ),
        )
        .unwrap();
    // Source paths changed: the previous acknowledgement must be refreshed.
    assert!(collect_sync_plan(&db, &options).is_err());
    let manifest = build_namespace_manifest(&db, &options).unwrap();
    assert_eq!(manifest.roots[0].root_uuid, original.root_uuid);
    assert_eq!(manifest.roots[0].legacy_prefix, original.legacy_prefix);
    let preview = save_namespace_manifest(dir.path(), manifest).unwrap();
    options.namespace_manifest_id = Some(preview.manifest_id);
    let keys: Vec<_> = plan
        .items
        .iter()
        .map(|item| item.remote_key.clone())
        .collect();
    let backup = dir.path().join("backup.db");
    let active = dir.path().join("omera.db");
    db.backup_database(backup.to_str().unwrap()).unwrap();
    drop(db);
    {
        let reopened = Database::connect(&active).unwrap();
        let current = collect_sync_plan(&reopened, &options).unwrap();
        assert_eq!(
            current
                .items
                .iter()
                .map(|item| item.remote_key.clone())
                .collect::<Vec<_>>(),
            keys
        );
        // Make active state differ, then restore the real SQLite snapshot.
        reopened
            .connection()
            .execute("DELETE FROM cloud_sync_roots", [])
            .unwrap();
    }
    omera_storage::recovery::stage_restore(&backup, &active).unwrap();
    omera_storage::recovery::apply_pending_restore(&active).unwrap();
    let restored = Database::connect(&active).unwrap();
    let current = collect_sync_plan(&restored, &options).unwrap();
    assert_eq!(
        current
            .items
            .iter()
            .map(|item| item.remote_key.clone())
            .collect::<Vec<_>>(),
        keys
    );
    for (item, content) in current.items.iter().zip([b"AAAA", b"BBBB"]) {
        assert_eq!(fs::read(&item.local_path).unwrap(), content);
    }
    assert!(backup.exists());
    assert!(dir.path().read_dir().unwrap().any(|entry| entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .contains("pre-restore")));
}
