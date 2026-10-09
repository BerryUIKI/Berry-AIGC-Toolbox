use omera_domain::{
    Container, ExportFormat, ExportOptions, ExportSidecar, ImageFile, ImportTransformRequest,
    LibraryTransformRequest, MetadataPrivacyMode, OriginalDisposition, TransformFormat,
    TransformSpec,
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

#[test]
fn standalone_avif_export_remains_available_but_cannot_reenter_managed_library() {
    let (dir, db, id, target) = fixture();
    let output = dir.path().join("export");
    let options = ExportOptions {
        file_ids: vec![id],
        format: ExportFormat::Avif,
        quality: 70,
        privacy: MetadataPrivacyMode::StripAll,
        sidecar: ExportSidecar::None,
        filename_template: "{name}".into(),
        destination_path: output.to_string_lossy().into_owned(),
        as_zip: false,
        max_edge: None,
        export_html_showcase: false,
        html_title: None,
    };
    let summary = omera_scan::execute_batch_export(&db, &options, |_| {});
    assert!(summary.success, "{:?}", summary.errors);
    let avif = output.join("source.avif");
    let bytes = fs::read(&avif).unwrap();
    assert_eq!(
        omera_metadata::detect_container(&bytes[..32]),
        Some(Container::Avif)
    );
    // This is intentionally not claimed as supported application decoding.
    let thumb = dir.path().join("unsupported-thumbnail.webp");
    assert!(omera_scan::thumbnail::generate_thumbnail(&avif, &thumb, 8).is_err());
    assert!(!thumb.exists());
    for filename in ["source.avif", "disguised.png"] {
        let source = output.join(filename);
        if source != avif {
            fs::write(&source, &bytes).unwrap();
        }
        let error = import_files_to_managed_folder(
            &db,
            &[source.to_string_lossy().into_owned()],
            target,
            None,
        )
        .unwrap_err();
        assert!(error.contains("export-only"));
        assert_eq!(fs::read(&source).unwrap(), bytes);
    }
    assert!(db.list_file_fingerprints(target).unwrap().is_empty());
}

#[test]
fn offered_managed_formats_reopen_thumbnail_and_reexport_with_shipped_codecs() {
    let (dir, db, id, target) = fixture();
    let source = db.get_file_by_id(id).unwrap().unwrap();
    for format in [
        TransformFormat::Png,
        TransformFormat::Jpeg,
        TransformFormat::Webp,
    ] {
        let spec = TransformSpec {
            format,
            max_edge: Some(8),
            ..Default::default()
        };
        let ids = import_files_to_managed_folder(
            &db,
            std::slice::from_ref(&source.path),
            target,
            Some(&spec),
        )
        .unwrap();
        let imported = db.get_file_by_id(ids[0]).unwrap().unwrap();
        let decoded = image::open(&imported.path).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (8, 4));
        let thumb = dir.path().join(format!("thumbnail-{format:?}.webp"));
        omera_scan::thumbnail::generate_thumbnail(Path::new(&imported.path), &thumb, 4).unwrap();
        let preview = image::open(&thumb).unwrap();
        assert_eq!((preview.width(), preview.height()), (4, 2));
        let options = ExportOptions {
            file_ids: ids,
            format: ExportFormat::Png,
            quality: 80,
            privacy: MetadataPrivacyMode::StripAll,
            sidecar: ExportSidecar::None,
            filename_template: "{name}".into(),
            destination_path: dir
                .path()
                .join(format!("reexport-{format:?}"))
                .to_string_lossy()
                .into_owned(),
            as_zip: false,
            max_edge: None,
            export_html_showcase: false,
            html_title: None,
        };
        let summary = omera_scan::execute_batch_export(&db, &options, |_| {});
        assert!(summary.success, "{:?}", summary.errors);
        let exported = fs::read_dir(&options.destination_path)
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let decoded = image::open(exported).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (8, 4));
    }
}
