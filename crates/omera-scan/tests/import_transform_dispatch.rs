use omera_domain::{Container, ImportTransformRequest, TransformMetadataPolicy, TransformSpec};
use omera_scan::{execute_managed_import_transform, import_files_to_managed_folder};
use omera_storage::Database;
use std::{fs, path::Path};
use tempfile::tempdir;

fn source_png(path: &Path) {
    let mut encoder = png::Encoder::new(fs::File::create(path).unwrap(), 65, 33);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.add_text_chunk("parameters".into(), "private prompt\nNegative prompt: secret exclusion\nSteps: 20, Seed: 42, Size: 65x33, Model: private-model".into()).unwrap();
    encoder
        .add_text_chunk("fixture_encoding_marker".into(), "copied original".into())
        .unwrap();
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&vec![120; 65 * 33 * 3])
        .unwrap();
}

#[test]
fn metadata_only_import_strips_source_bytes_through_both_entry_points() {
    for job_entry_point in [false, true] {
        for policy in [
            TransformMetadataPolicy::StripAi,
            TransformMetadataPolicy::StripAll,
        ] {
            let dir = tempdir().unwrap();
            let source = dir.path().join("source.png");
            source_png(&source);
            let before = fs::read(&source).unwrap();
            fs::write(source.with_extension("txt"), "private sidecar prompt").unwrap();
            fs::write(source.with_extension("json"), r#"{"prompt":"private"}"#).unwrap();
            let vault = dir.path().join("vault");
            fs::create_dir(&vault).unwrap();
            let db = Database::connect_in_memory().unwrap();
            let folder = db
                .add_folder_with_mode(&vault.to_string_lossy(), "managed", None, None, None, true)
                .unwrap();
            let spec = TransformSpec {
                metadata_policy: policy,
                ..Default::default()
            };
            if job_entry_point {
                let receipt = execute_managed_import_transform(
                    &db,
                    &ImportTransformRequest {
                        managed_destination_id: folder.id,
                        source_paths: vec![source.to_string_lossy().into_owned()],
                        spec,
                        source_disposition: Default::default(),
                    },
                    None::<fn(usize, usize, &str)>,
                )
                .unwrap();
                assert_eq!(receipt.succeeded, 1);
            } else {
                assert_eq!(
                    import_files_to_managed_folder(
                        &db,
                        &[source.to_string_lossy().into_owned()],
                        folder.id,
                        Some(&spec)
                    )
                    .unwrap()
                    .len(),
                    1
                );
            }
            let output = vault.join("source.png");
            assert!(
                omera_metadata::extract_metadata(Container::Png, &output).is_none(),
                "{policy:?}, job={job_entry_point}"
            );
            assert!(db.list_files(folder.id).unwrap()[0]
                .metadata
                .as_ref()
                .is_none_or(|m| m.prompt.is_none()));
            assert!(!output.with_extension("txt").exists());
            assert!(!output.with_extension("json").exists());
            assert_eq!(image::image_dimensions(output).unwrap(), (65, 33));
            assert_eq!(fs::read(&source).unwrap(), before);
        }
    }
}

#[test]
fn unsupported_video_stripping_cannot_succeed_via_raw_copy() {
    for extension in ["mp4", "webm"] {
        for policy in [
            TransformMetadataPolicy::StripAi,
            TransformMetadataPolicy::StripAll,
        ] {
            let dir = tempdir().unwrap();
            let source = dir.path().join(format!("source.{extension}"));
            fs::write(&source, "synthetic video fixture").unwrap();
            let vault = dir.path().join("vault");
            fs::create_dir(&vault).unwrap();
            let db = Database::connect_in_memory().unwrap();
            let folder = db
                .add_folder_with_mode(&vault.to_string_lossy(), "managed", None, None, None, true)
                .unwrap();
            let error = import_files_to_managed_folder(
                &db,
                &[source.to_string_lossy().into_owned()],
                folder.id,
                Some(&TransformSpec {
                    metadata_policy: policy,
                    ..Default::default()
                }),
            )
            .unwrap_err();
            assert!(error.contains("unsupported"));
            assert_eq!(fs::read_dir(vault).unwrap().count(), 0);
            assert!(db.list_files(folder.id).unwrap().is_empty());
            assert_eq!(fs::read(source).unwrap(), b"synthetic video fixture");
        }
    }
}
