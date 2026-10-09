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
