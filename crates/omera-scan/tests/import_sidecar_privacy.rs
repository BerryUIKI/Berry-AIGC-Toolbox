use omera_domain::{Container, TransformFormat, TransformMetadataPolicy, TransformSpec};
use omera_scan::import_files_to_managed_folder;
use omera_storage::Database;
use std::fs;
use tempfile::tempdir;

#[test]
fn import_sidecars_obey_policy_with_and_without_reencoding() {
    for reencode in [false, true] {
        for policy in [
            TransformMetadataPolicy::KeepSupported,
            TransformMetadataPolicy::StripAi,
            TransformMetadataPolicy::StripAll,
        ] {
            let dir = tempdir().unwrap();
            let source = dir.path().join("source.png");
            image::RgbImage::new(16, 8).save(&source).unwrap();
            let text = b"private prompt\nNegative prompt: private exclusion\nSteps: 20, Seed: 42";
            let json = br#"{"prompt":"private prompt","workflow":{"private":"secret"}}"#;
            fs::write(source.with_extension("txt"), text).unwrap();
            fs::write(source.with_extension("json"), json).unwrap();
            let destination = dir.path().join("vault");
            fs::create_dir(&destination).unwrap();
            let db = Database::connect_in_memory().unwrap();
            let folder = db
                .add_folder_with_mode(
                    &destination.to_string_lossy(),
                    "managed",
                    None,
                    None,
                    None,
                    true,
                )
                .unwrap();
            let spec = TransformSpec {
                format: if reencode {
                    TransformFormat::Png
                } else {
                    TransformFormat::Original
                },
                max_edge: reencode.then_some(8),
                metadata_policy: policy,
                ..Default::default()
            };
            let ids = import_files_to_managed_folder(
                &db,
                &[source.to_string_lossy().into_owned()],
                folder.id,
                Some(&spec),
            )
            .unwrap();
            assert_eq!(ids.len(), 1);
            let file = db.get_file_by_id(ids[0]).unwrap().unwrap();
            let output = std::path::Path::new(&file.path);
            if policy == TransformMetadataPolicy::KeepSupported {
                assert_eq!(fs::read(output.with_extension("txt")).unwrap(), text);
                assert_eq!(fs::read(output.with_extension("json")).unwrap(), json);
                assert!(omera_metadata::extract_metadata(Container::Png, output)
                    .unwrap()
                    .prompt
                    .is_some());
            } else {
                assert!(
                    !output.with_extension("txt").exists(),
                    "{policy:?}, reencode={reencode}"
                );
                assert!(!output.with_extension("json").exists());
                assert!(omera_metadata::extract_metadata(Container::Png, output).is_none());
                assert!(file.metadata.as_ref().is_none_or(|m| m.prompt.is_none()));
            }
            assert_eq!(fs::read(source.with_extension("txt")).unwrap(), text);
            assert_eq!(fs::read(source.with_extension("json")).unwrap(), json);
        }
    }
}
