use omera_domain::{
    Container, ImageFile, ImportTransformRequest, LibraryTransformRequest, OriginalDisposition,
    TransformFormat, TransformSpec,
};
use omera_scan::{
    execute_library_batch_transform, execute_managed_import_transform,
    import_files_to_managed_folder,
};
use omera_storage::Database;
use std::{fs, path::Path};
use tempfile::{tempdir, TempDir};

fn fixture() -> (TempDir, Database, i64, i64) {
    let dir = tempdir().unwrap();
    let source = dir.path().join("source.png");
    image::RgbImage::from_pixel(16, 8, image::Rgb([40, 80, 120]))
        .save(&source)
        .unwrap();
    let db = Database::connect_in_memory().unwrap();
    let folder = db
        .add_folder_with_mode(
            &dir.path().to_string_lossy(),
            "managed",
            None,
            None,
            None,
            true,
        )
        .unwrap();
    let id = db
        .upsert_file(&ImageFile {
            id: None,
            folder_id: folder.id,
            path: source.to_string_lossy().into_owned(),
            container: Container::Png,
            size_bytes: fs::metadata(&source).unwrap().len(),
            modified_at: 1,
            metadata: None,
            rating: Some(8),
            aesthetic_score: Some(6.0),
            is_favorite: true,
            is_nsfw: false,
            stack_id: Some("curated".into()),
            stack_order: 1,
        })
        .unwrap();
    let vault = dir.path().join("vault");
    fs::create_dir(&vault).unwrap();
    let target = db
        .add_folder_with_mode(&vault.to_string_lossy(), "managed", None, None, None, true)
        .unwrap();
    (dir, db, id, target.id)
}

#[test]
fn avif_target_is_rejected_by_both_import_entry_points_and_library_batch() {
    let (dir, db, id, target) = fixture();
    let before = db.get_file_by_id(id).unwrap().unwrap();
    let bytes = fs::read(&before.path).unwrap();
    let spec = TransformSpec {
        format: TransformFormat::Avif,
        ..Default::default()
    };
    let paths = vec![before.path.clone()];
    let error = import_files_to_managed_folder(&db, &paths, target, Some(&spec)).unwrap_err();
    assert!(error.contains("export-only"));
    let import = execute_managed_import_transform(
        &db,
        &ImportTransformRequest {
            source_paths: paths,
            managed_destination_id: target,
            spec: spec.clone(),
            source_disposition: Default::default(),
        },
        None::<fn(usize, usize, &str)>,
    )
    .unwrap();
    assert_eq!((import.succeeded, import.failed), (0, 1));
    let batch = execute_library_batch_transform(
        &db,
        &LibraryTransformRequest {
            file_ids: vec![id],
            spec,
            original_disposition: OriginalDisposition::Archive,
        },
        None::<fn(usize, usize, &str)>,
    )
    .unwrap();
    assert_eq!((batch.succeeded, batch.failed), (0, 1));
    assert!(batch.items[0]
        .error_code
        .as_ref()
        .unwrap()
        .contains("export-only"));
    assert_eq!(db.get_file_by_id(id).unwrap().unwrap(), before);
    assert_eq!(fs::read(&before.path).unwrap(), bytes);
    assert!(db.list_file_fingerprints(target).unwrap().is_empty());
    assert!(!dir.path().join(".omera_archive").exists());
    assert!(dir
        .path()
        .join("vault")
        .read_dir()
        .unwrap()
        .all(|entry| entry.unwrap().path().is_dir()));
}

#[test]
fn malformed_avif_source_is_rejected_without_publication() {
    let (dir, db, _, target) = fixture();
    let source = dir.path().join("malformed.avif");
    fs::write(&source, b"malformed AVIF fixture").unwrap();
    let error =
        import_files_to_managed_folder(&db, &[source.to_string_lossy().into_owned()], target, None)
            .unwrap_err();
    assert!(error.contains("export-only"));
    assert_eq!(fs::read(&source).unwrap(), b"malformed AVIF fixture");
    assert!(db.list_file_fingerprints(target).unwrap().is_empty());
    assert!(!Path::new(&source)
        .with_file_name("vault/malformed.avif")
        .exists());
}
