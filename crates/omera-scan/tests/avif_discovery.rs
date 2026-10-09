use omera_domain::{
    Container, ExportFormat, ExportOptions, ExportSidecar, ImageFile, MetadataPrivacyMode,
};
use omera_scan::Scanner;
use omera_storage::Database;
use std::{fs, path::PathBuf};
use tempfile::{tempdir, TempDir};

fn exported_fixture() -> (TempDir, Database, PathBuf, PathBuf, i64) {
    let dir = tempdir().unwrap();
    let source_dir = dir.path().join("source");
    fs::create_dir(&source_dir).unwrap();
    let source = source_dir.join("image.png");
    image::RgbImage::from_pixel(32, 16, image::Rgb([90, 120, 170]))
        .save(&source)
        .unwrap();
    let database = dir.path().join("library.db");
    let db = Database::connect(&database).unwrap();
    let folder = db.add_folder(&source_dir.to_string_lossy()).unwrap();
    let id = db
        .upsert_file(&ImageFile {
            id: None,
            folder_id: folder.id,
            path: source.to_string_lossy().into_owned(),
            size_bytes: fs::metadata(&source).unwrap().len(),
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
    let exports = dir.path().join("exports");
    let summary = omera_scan::execute_batch_export(
        &db,
        &ExportOptions {
            file_ids: vec![id],
            format: ExportFormat::Avif,
            quality: 70,
            privacy: MetadataPrivacyMode::StripAll,
            sidecar: ExportSidecar::None,
            filename_template: "{name}".into(),
            destination_path: exports.to_string_lossy().into_owned(),
            as_zip: false,
            max_edge: None,
            export_html_showcase: false,
            html_title: None,
        },
        |_| {},
    );
    assert!(summary.success, "{:?}", summary.errors);
    let target = db.add_folder(&exports.to_string_lossy()).unwrap();
    (dir, db, database, exports, target.id)
}

#[test]
fn app_exported_avif_only_directory_is_indexed_without_claiming_decoder_support() {
    let (dir, db, database, exports, folder) = exported_fixture();
    let avif = exports.join("image.avif");
    let original = fs::read(&avif).unwrap();
    let source = fs::read(dir.path().join("source/image.png")).unwrap();
    let scanner = Scanner::with_default_extractor(database);
    let stats = scanner.scan_folder(folder, &exports, |_| {}).unwrap();
    assert_eq!((stats.found, stats.added, stats.failed), (1, 1, 0));
    let indexed = db.list_files(folder).unwrap();
    assert_eq!(indexed.len(), 1);
    assert_eq!(indexed[0].container, Container::Avif);
    assert_eq!(
        serde_json::to_value(&indexed[0]).unwrap()["container"],
        "Avif"
    );
    let preview = dir.path().join("preview.webp");
    assert!(omera_scan::thumbnail::generate_thumbnail(&avif, &preview, 8).is_err());
    assert!(!preview.exists());
    assert_eq!(fs::read(&avif).unwrap(), original);
    assert_eq!(
        fs::read(dir.path().join("source/image.png")).unwrap(),
        source
    );
    let again = scanner.scan_folder(folder, &exports, |_| {}).unwrap();
    assert_eq!((again.found, again.added, again.failed), (1, 0, 0));
    assert_eq!(db.list_files(folder).unwrap()[0].id, indexed[0].id);
}

#[test]
fn mixed_avif_scan_uses_magic_bytes_and_rejects_unrecognized_input() {
    let (dir, db, database, exports, folder) = exported_fixture();
    fs::copy(
        dir.path().join("source/image.png"),
        exports.join("image.png"),
    )
    .unwrap();
    fs::copy(exports.join("image.avif"), exports.join("upper.AVIF")).unwrap();
    // Extension does not override recognizable PNG bytes.
    fs::copy(exports.join("image.png"), exports.join("actually-png.avif")).unwrap();
    fs::write(exports.join("malformed.avif"), b"not an image").unwrap();
    fs::write(exports.join("truncated.avif"), b"\x00\x00\x00\x18ftyp").unwrap();
    let stats = Scanner::with_default_extractor(database)
        .scan_folder(folder, &exports, |_| {})
        .unwrap();
    assert_eq!((stats.found, stats.added, stats.failed), (6, 4, 2));
    let indexed = db.list_files(folder).unwrap();
    assert_eq!(
        indexed
            .iter()
            .filter(|file| file.container == Container::Avif)
            .count(),
        2
    );
    assert_eq!(
        indexed
            .iter()
            .filter(|file| file.container == Container::Png)
            .count(),
        2
    );
    assert!(
        !indexed
            .iter()
            .any(|file| file.path.ends_with("malformed.avif")
                || file.path.ends_with("truncated.avif"))
    );
}
