#[path = "../../../../src-tauri/src/cloud_backup.rs"]
mod cloud_backup;

#[cfg(test)]
mod probes {
    use omera_domain::*;
    use omera_storage::Database;
    use std::{fs, path::Path};

    fn make_png(path: &Path) {
        let writer = fs::File::create(path).unwrap();
        let mut encoder = png::Encoder::new(writer, 64, 32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.add_text_chunk("parameters".into(), "confidential prompt\nNegative prompt: private negative\nSteps: 20, Sampler: Euler, CFG scale: 7, Seed: 123, Size: 64x32, Model: private-model".into()).unwrap();
        encoder.write_header().unwrap().write_image_data(&vec![120; 64 * 32 * 3]).unwrap();
    }

    fn make_file(path: &Path, folder_id: i64) -> ImageFile {
        ImageFile {
            id: None, folder_id, path: path.to_string_lossy().into_owned(),
            size_bytes: fs::metadata(path).unwrap().len(), modified_at: 1,
            container: Container::Png, metadata: omera_metadata::extract_metadata(Container::Png, path),
            rating: Some(4), aesthetic_score: None, is_favorite: true, is_nsfw: false,
            stack_id: None, stack_order: 0,
        }
    }

    #[test]
    fn negative_prompt_exclusions_must_not_classify_a_safe_prompt_as_nsfw() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("safe-landscape.png");
        let mut encoder = png::Encoder::new(fs::File::create(&path).unwrap(), 16, 16);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.add_text_chunk("parameters".into(), "landscape, mountain, daylight\nNegative prompt: nsfw, nude\nSteps: 20, Sampler: Euler, Seed: 123, Size: 16x16".into()).unwrap();
        encoder.write_header().unwrap().write_image_data(&vec![120; 16 * 16 * 3]).unwrap();
        let metadata = omera_metadata::extract_metadata(Container::Png, &path).unwrap();
        assert_eq!(metadata.prompt.as_deref(), Some("landscape, mountain, daylight"));
        assert!(!omera_metadata::detect_nsfw_from_metadata(&metadata), "Excluded negative-prompt terms classified a safe positive prompt as NSFW");
    }

    fn export_options(privacy: MetadataPrivacyMode) -> ExportOptions {
        ExportOptions {
            file_ids: vec![], format: ExportFormat::Png, quality: 85, privacy,
            sidecar: ExportSidecar::TextPrompt, filename_template: "{name}".into(),
            destination_path: "unused".into(), as_zip: false, max_edge: None,
            export_html_showcase: false, html_title: None,
        }
    }

    #[test]
    fn stripping_prompt_must_not_emit_prompt_text_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let file = make_file(&source, 1);
        for policy in [MetadataPrivacyMode::StripPromptOnly, MetadataPrivacyMode::StripAllAiMetadata] {
            let output = omera_scan::export::process_single_image(&file, &export_options(policy), 0).unwrap();
            assert!(output.sidecar.is_none(), "Privacy policy {policy:?} emitted a confidential prompt sidecar");
        }
    }

    #[test]
    fn strip_ai_metadata_must_not_emit_prompt_text_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let output = omera_scan::export::process_single_image(&make_file(&source, 1), &export_options(MetadataPrivacyMode::StripAllAiMetadata), 0).unwrap();
        assert!(output.sidecar.is_none(), "StripAllAiMetadata emitted a confidential prompt sidecar");
    }

    #[test]
    fn upsert_existing_path_must_return_its_actual_id() {
        let dir = tempfile::tempdir().unwrap();
        let source_a = dir.path().join("a.png"); make_png(&source_a);
        let source_b = dir.path().join("b.png"); make_png(&source_b);
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder(dir.path().to_str().unwrap()).unwrap();
        let file_a = make_file(&source_a, folder.id);
        let id_a = db.upsert_file(&file_a).unwrap();
        let id_b = db.upsert_file(&make_file(&source_b, folder.id)).unwrap();
        assert_ne!(id_a, id_b);
        assert_eq!(db.upsert_file(&file_a).unwrap(), id_a, "Updating a.png returns the unrelated b.png ID");
    }

    #[test]
    fn cloud_restore_must_survive_active_wal_connection() {
        let dir = tempfile::tempdir().unwrap();
        let active_path = dir.path().join("active.db");
        let active = Database::connect(&active_path).unwrap();
        active.add_folder("active-before-checkpoint").unwrap();
        active.connection().execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").unwrap();
        active.add_folder("active-in-wal").unwrap();
        let snapshot_db = Database::connect_in_memory().unwrap();
        snapshot_db.add_folder("restored-snapshot").unwrap();
        let cloud = dir.path().join("cloud");
        let config = CloudBackupConfig { local_path: Some(cloud.to_string_lossy().into_owned()), ..Default::default() };
        let created = crate::cloud_backup::create_cloud_snapshot(&snapshot_db, &config, dir.path(), None).unwrap();
        let filename = created.snapshot.unwrap().filename;
        let restored = crate::cloud_backup::restore_cloud_snapshot(&active_path, &config, &filename).unwrap();
        assert!(restored.success);
        drop(active);
        let reopened = Database::connect(&active_path).unwrap();
        let folders = reopened.list_folders().unwrap();
        assert_eq!(folders.len(), 1, "Cloud restore reported success, but active WAL overwrote the restored data: {folders:?}");
        assert_eq!(folders[0].path, "restored-snapshot");
    }

    #[test]
    fn keep_supported_transform_must_preserve_embedded_prompt() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let spec = TransformSpec { format: TransformFormat::Png, max_edge: Some(32), ..Default::default() };
        let output = omera_scan::transform_file_staged(&source, &dir.path().join("stage"), &spec).unwrap();
        let parsed = omera_metadata::extract_metadata(Container::Png, &output);
        assert_eq!(parsed.and_then(|m| m.prompt), Some("confidential prompt".into()));
    }

    #[test]
    fn exported_avif_must_be_decodable_for_library_thumbnail() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let spec = TransformSpec { format: TransformFormat::Avif, ..Default::default() };
        let output = omera_scan::transform_file_staged(&source, &dir.path().join("stage"), &spec).unwrap();
        let result = omera_scan::thumbnail::generate_thumbnail(&output, &dir.path().join("thumb.webp"), 32);
        assert!(result.is_ok(), "Created AVIF fails library thumbnail decode: {result:?}");
    }

    #[test]
    fn managed_transform_import_must_reject_linked_destination() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let dest = dir.path().join("external"); fs::create_dir(&dest).unwrap();
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder(dest.to_str().unwrap()).unwrap();
        let request = ImportTransformRequest { managed_destination_id: folder.id,
            source_paths: vec![source.to_str().unwrap().into()], spec: TransformSpec::default(),
            source_disposition: ImportSourceDisposition::Keep };
        let result = omera_scan::execute_managed_import_transform(&db, &request, None::<fn(usize, usize, &str)>);
        assert!(result.is_err(), "Managed import writes successfully into a read-only linked folder: {result:?}");
    }

    #[test]
    fn archive_must_wait_until_database_update_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder_with_mode(dir.path().to_str().unwrap(), "managed", None, None, None, false).unwrap();
        let file = make_file(&source, folder.id);
        let id = db.upsert_file(&file).unwrap();
        db.connection().execute_batch("CREATE TRIGGER review_fail_update BEFORE UPDATE ON files BEGIN SELECT RAISE(FAIL, 'review simulated database write failure'); END;").unwrap();
        let request = LibraryTransformRequest { file_ids: vec![id], spec: TransformSpec { format: TransformFormat::Jpeg, ..Default::default() }, original_disposition: OriginalDisposition::Archive };
        let result = omera_scan::execute_library_batch_transform(&db, &request, None::<fn(usize, usize, &str)>).unwrap();
        assert_eq!(result.failed, 1);
        assert!(source.exists(), "Source was archived before a failed DB update; indexed source path is now missing");
    }

    #[test]
    fn transformed_database_dimensions_must_match_derivative() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder_with_mode(dir.path().to_str().unwrap(), "managed", None, None, None, false).unwrap();
        let id = db.upsert_file(&make_file(&source, folder.id)).unwrap();
        let request = LibraryTransformRequest { file_ids: vec![id], spec: TransformSpec { format: TransformFormat::Jpeg, max_edge: Some(32), ..Default::default() }, original_disposition: OriginalDisposition::Keep };
        let result = omera_scan::execute_library_batch_transform(&db, &request, None::<fn(usize, usize, &str)>).unwrap();
        assert_eq!(result.succeeded, 1);
        let file = db.get_file_by_id(id).unwrap().unwrap();
        assert_eq!(file.metadata.unwrap().width, Some(32), "Inspector retains original dimensions after successful resize");
    }

    #[test]
    fn strip_all_batch_transform_must_not_retain_prompt_in_library() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder_with_mode(dir.path().to_str().unwrap(), "managed", None, None, None, false).unwrap();
        let id = db.upsert_file(&make_file(&source, folder.id)).unwrap();
        let request = LibraryTransformRequest { file_ids: vec![id], spec: TransformSpec { format: TransformFormat::Jpeg, metadata_policy: TransformMetadataPolicy::StripAll, ..Default::default() }, original_disposition: OriginalDisposition::Keep };
        let result = omera_scan::execute_library_batch_transform(&db, &request, None::<fn(usize, usize, &str)>).unwrap();
        assert_eq!(result.succeeded, 1);
        let metadata = db.get_file_by_id(id).unwrap().unwrap().metadata;
        assert!(metadata.as_ref().and_then(|m| m.prompt.as_ref()).is_none(), "strip_all transform still exposes original private prompt in library");
    }
}
