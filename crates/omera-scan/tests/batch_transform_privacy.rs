use omera_domain::{
    Container, ImageFile, LibraryTransformRequest, OriginalDisposition, TransformFormat,
    TransformMetadataPolicy, TransformSpec,
};
use omera_scan::execute_library_batch_transform;
use omera_storage::Database;
use std::{fs, path::Path};
use tempfile::{tempdir, TempDir};

fn fixture() -> (TempDir, Database, i64) {
    let dir = tempdir().unwrap();
    let source = dir.path().join("source.png");
    let mut encoder = png::Encoder::new(fs::File::create(&source).unwrap(), 16, 8);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.add_text_chunk("parameters".into(), "private source prompt\nNegative prompt: private exclusion\nSteps: 20, Seed: 42, Model: private-model".into()).unwrap();
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&[120; 16 * 8 * 3])
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
    let file = ImageFile {
        id: None,
        folder_id: folder.id,
        path: source.to_string_lossy().into_owned(),
        size_bytes: fs::metadata(&source).unwrap().len(),
        modified_at: 1,
        container: Container::Png,
        metadata: omera_metadata::extract_metadata(Container::Png, &source),
        rating: Some(8),
        aesthetic_score: Some(6.5),
        is_favorite: true,
        is_nsfw: true,
        stack_id: Some("curated-stack".into()),
        stack_order: 2,
    };
    let id = db.upsert_file(&file).unwrap();
    let tag = db.create_tag("curated", None).unwrap();
    db.tag_file(id, tag.id).unwrap();
    let album = db.create_album("curated", None).unwrap();
    db.add_file_to_album(album.id, id).unwrap();
    (dir, db, id)
}

#[test]
fn batch_transform_privacy_matches_bytes_without_losing_curation() {
    for policy in [
        TransformMetadataPolicy::KeepSupported,
        TransformMetadataPolicy::StripAi,
        TransformMetadataPolicy::StripAll,
    ] {
        let (dir, db, id) = fixture();
        let before = db.get_file_by_id(id).unwrap().unwrap();
        let source_bytes = fs::read(&before.path).unwrap();
        let request = LibraryTransformRequest {
            file_ids: vec![id],
            spec: TransformSpec {
                format: TransformFormat::Png,
                max_edge: Some(8),
                metadata_policy: policy,
                ..Default::default()
            },
            original_disposition: OriginalDisposition::Keep,
        };
        let receipt =
            execute_library_batch_transform(&db, &request, None::<fn(usize, usize, &str)>).unwrap();
        assert_eq!(receipt.succeeded, 1);
        let after = db.get_file_by_id(id).unwrap().unwrap();
        let output_metadata =
            omera_metadata::extract_metadata(Container::Png, Path::new(&after.path));
        assert_eq!(after.metadata, output_metadata, "{policy:?}");
        if policy == TransformMetadataPolicy::KeepSupported {
            assert_eq!(
                after.metadata.as_ref().unwrap().prompt.as_deref(),
                Some("private source prompt")
            );
        } else {
            assert!(after.metadata.is_none(), "{policy:?}");
            let stored: Option<String> = db
                .connection()
                .query_row("SELECT metadata FROM files WHERE id = ?1", [id], |r| {
                    r.get(0)
                })
                .unwrap();
            assert!(stored.is_none());
        }
        assert_eq!(after.rating, before.rating);
        assert_eq!(after.aesthetic_score, before.aesthetic_score);
        assert!(after.is_favorite && after.is_nsfw);
        assert_eq!(
            (after.stack_id, after.stack_order),
            (before.stack_id, before.stack_order)
        );
        assert_eq!(db.get_file_tags(id).unwrap()[0].name, "curated");
        assert_eq!(db.list_album_files(1).unwrap()[0].id, Some(id));
        assert_eq!(fs::read(&before.path).unwrap(), source_bytes);
        assert!(dir.path().join("source_1.png").exists());
    }
}
