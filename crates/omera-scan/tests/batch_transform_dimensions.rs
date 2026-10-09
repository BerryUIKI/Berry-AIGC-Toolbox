use image::GenericImageView;
use omera_domain::{
    Container, ImageFile, LibraryTransformRequest, OriginalDisposition, SearchCriteria,
    TransformFormat, TransformMetadataPolicy, TransformSpec,
};
use omera_scan::execute_library_batch_transform;
use omera_storage::Database;
use std::fs;
use tempfile::tempdir;

#[test]
fn decoded_dimensions_survive_batch_transform_gallery_query_and_restart() {
    let cases = [
        (
            64,
            32,
            TransformSpec {
                max_edge: Some(32),
                ..Default::default()
            },
            (32, 16),
        ),
        (
            64,
            32,
            TransformSpec {
                scale_percent: Some(75),
                ..Default::default()
            },
            (48, 24),
        ),
        (
            66,
            34,
            TransformSpec {
                align_multiple: Some(16),
                ..Default::default()
            },
            (64, 32),
        ),
        (
            64,
            32,
            TransformSpec {
                format: TransformFormat::Jpeg,
                ..Default::default()
            },
            (64, 32),
        ),
        (
            64,
            32,
            TransformSpec {
                format: TransformFormat::Webp,
                ..Default::default()
            },
            (64, 32),
        ),
        (
            32,
            64,
            TransformSpec {
                max_edge: Some(32),
                ..Default::default()
            },
            (16, 32),
        ),
    ];
    for (width, height, spec, expected) in cases {
        for policy in [
            TransformMetadataPolicy::KeepSupported,
            TransformMetadataPolicy::StripAi,
            TransformMetadataPolicy::StripAll,
        ] {
            let dir = tempdir().unwrap();
            let assets = dir.path().join("assets");
            fs::create_dir(&assets).unwrap();
            let source = assets.join("source.png");
            let mut encoder = png::Encoder::new(fs::File::create(&source).unwrap(), width, height);
            encoder.set_color(png::ColorType::Rgb);
            encoder.set_depth(png::BitDepth::Eight);
            // Raw generation dimensions deliberately differ from actual pixels.
            encoder
                .add_text_chunk(
                    "parameters".into(),
                    "source prompt\nSteps: 20, Seed: 42, Size: 4096x2048".into(),
                )
                .unwrap();
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&vec![90; (width * height * 3) as usize])
                .unwrap();
            let source_bytes = fs::read(&source).unwrap();
            let database_path = dir.path().join("library.db");
            let db = Database::connect(&database_path).unwrap();
            let folder = db
                .add_folder_with_mode(&assets.to_string_lossy(), "managed", None, None, None, true)
                .unwrap();
            let id = db
                .upsert_file(&ImageFile {
                    id: None,
                    folder_id: folder.id,
                    path: source.to_string_lossy().into_owned(),
                    size_bytes: source_bytes.len() as u64,
                    modified_at: 1,
                    container: Container::Png,
                    metadata: omera_metadata::extract_metadata(Container::Png, &source),
                    rating: Some(8),
                    aesthetic_score: Some(7.5),
                    is_favorite: true,
                    is_nsfw: true,
                    stack_id: Some("same-stack".into()),
                    stack_order: 3,
                })
                .unwrap();
            let before = db.get_file_by_id(id).unwrap().unwrap();
            assert_eq!(before.metadata.unwrap().width, Some(4096));
            let tag = db.create_tag("curated", None).unwrap();
            db.tag_file(id, tag.id).unwrap();
            let album = db.create_album("curated", None).unwrap();
            db.add_file_to_album(album.id, id).unwrap();
            let receipt = execute_library_batch_transform(
                &db,
                &LibraryTransformRequest {
                    file_ids: vec![id],
                    spec: TransformSpec {
                        metadata_policy: policy,
                        ..spec.clone()
                    },
                    original_disposition: OriginalDisposition::Keep,
                },
                None::<fn(usize, usize, &str)>,
            )
            .unwrap();
            assert_eq!(
                (receipt.succeeded, receipt.failed),
                (1, 0),
                "{spec:?}/{policy:?}"
            );
            let output = db.get_file_by_id(id).unwrap().unwrap();
            assert_eq!(image::open(&output.path).unwrap().dimensions(), expected);
            let metadata = output.metadata.as_ref().unwrap();
            assert_eq!(
                (metadata.width, metadata.height),
                (Some(expected.0), Some(expected.1)),
                "{spec:?}/{policy:?}"
            );
            if policy == TransformMetadataPolicy::KeepSupported
                && output.container == Container::Png
            {
                assert!(metadata
                    .parameters
                    .as_ref()
                    .unwrap()
                    .contains("Size: 4096x2048"));
                assert_eq!(metadata.prompt.as_deref(), Some("source prompt"));
            } else {
                assert!(
                    metadata.prompt.is_none()
                        && metadata.parameters.is_none()
                        && metadata.raw.is_none()
                );
                assert_eq!(
                    serde_json::to_value(metadata).unwrap()["format"],
                    serde_json::Value::Null
                );
            }
            assert_eq!(fs::read(&source).unwrap(), source_bytes);
            drop(db);
            let restarted = Database::connect(&database_path).unwrap();
            assert_eq!(restarted.get_file_by_id(id).unwrap().unwrap(), output);
            let page = restarted
                .search_files_page(&SearchCriteria {
                    limit: Some(10),
                    ..Default::default()
                })
                .unwrap();
            let gallery = page.items.iter().find(|file| file.id == Some(id)).unwrap();
            let geometry = gallery.metadata.as_ref().unwrap();
            assert_eq!(
                (geometry.width, geometry.height),
                (Some(expected.0), Some(expected.1))
            );
            assert_eq!(
                (
                    gallery.rating,
                    gallery.aesthetic_score,
                    gallery.is_favorite,
                    gallery.is_nsfw
                ),
                (Some(8), Some(7.5), true, true)
            );
            assert_eq!(
                (gallery.stack_id.as_deref(), gallery.stack_order),
                (Some("same-stack"), 3)
            );
            assert_eq!(restarted.get_file_tags(id).unwrap()[0].name, "curated");
            assert_eq!(
                restarted.list_album_files(album.id).unwrap()[0].id,
                Some(id)
            );
        }
    }
}
