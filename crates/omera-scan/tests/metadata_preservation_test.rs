use omera_domain::{Container, TransformFormat, TransformMetadataPolicy, TransformSpec};
use omera_scan::transform_file_staged;
use std::fs;
use tempfile::tempdir;

fn create_png_with_metadata(path: &std::path::Path, width: u32, height: u32, prompt: &str) {
    let file = fs::File::create(path).unwrap();
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .add_text_chunk("parameters".into(), prompt.into())
        .unwrap();
    encoder
        .write_header()
        .unwrap()
        .write_image_data(&vec![120; (width * height * 3) as usize])
        .unwrap();
}

#[test]
fn test_keep_supported_preserves_png_metadata() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("source.png");
    let prompt = "a magical forest\nNegative prompt: blurry\nSteps: 20, Sampler: Euler, CFG scale: 7, Seed: 12345, Size: 64x32, Model: test-model";

    create_png_with_metadata(&source, 64, 32, prompt);

    let spec = TransformSpec {
        format: TransformFormat::Png,
        max_edge: Some(32),
        metadata_policy: TransformMetadataPolicy::KeepSupported,
        ..Default::default()
    };

    let staging = dir.path().join("staging");
    let output = transform_file_staged(&source, &staging, &spec).unwrap();

    // Verify metadata is preserved in the output
    let metadata = omera_metadata::extract_metadata(Container::Png, &output);
    assert!(
        metadata.is_some(),
        "Metadata should be present in transformed PNG"
    );

    let meta = metadata.unwrap();
    assert_eq!(meta.prompt, Some("a magical forest".to_string()));
    assert_eq!(meta.negative_prompt, Some("blurry".to_string()));
}

#[test]
fn test_strip_ai_removes_generation_metadata() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("source.png");
    let prompt = "a magical forest\nNegative prompt: blurry\nSteps: 20, Sampler: Euler";

    create_png_with_metadata(&source, 64, 32, prompt);

    let spec = TransformSpec {
        format: TransformFormat::Png,
        max_edge: Some(32),
        metadata_policy: TransformMetadataPolicy::StripAi,
        ..Default::default()
    };

    let staging = dir.path().join("staging");
    let output = transform_file_staged(&source, &staging, &spec).unwrap();

    // Verify AI metadata is stripped
    let metadata = omera_metadata::extract_metadata(Container::Png, &output);
    assert!(
        metadata.is_none() || metadata.unwrap().prompt.is_none(),
        "AI generation metadata should be stripped"
    );
}

#[test]
fn test_strip_all_removes_all_metadata() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("source.png");
    let prompt = "a magical forest\nSteps: 20, Sampler: Euler";

    create_png_with_metadata(&source, 64, 32, prompt);

    let spec = TransformSpec {
        format: TransformFormat::Png,
        max_edge: Some(32),
        metadata_policy: TransformMetadataPolicy::StripAll,
        ..Default::default()
    };

    let staging = dir.path().join("staging");
    let output = transform_file_staged(&source, &staging, &spec).unwrap();

    // Verify all metadata is stripped
    let metadata = omera_metadata::extract_metadata(Container::Png, &output);
    assert!(metadata.is_none(), "All metadata should be stripped");
}

#[test]
fn test_jpeg_transform_documents_limitation() {
    let dir = tempdir().unwrap();
    let source = dir.path().join("source.png");
    let prompt = "a magical forest\nSteps: 20";

    create_png_with_metadata(&source, 64, 32, prompt);

    let spec = TransformSpec {
        format: TransformFormat::Jpeg,
        quality: Some(85),
        metadata_policy: TransformMetadataPolicy::KeepSupported,
        ..Default::default()
    };

    let staging = dir.path().join("staging");
    let output = transform_file_staged(&source, &staging, &spec).unwrap();

    // JPEG does not support embedded prompt preservation in current implementation
    // This test documents the current limitation
    let metadata = omera_metadata::extract_metadata(Container::Jpeg, &output);
    assert!(
        metadata.is_none() || metadata.unwrap().prompt.is_none(),
        "JPEG metadata preservation not yet implemented"
    );
}
