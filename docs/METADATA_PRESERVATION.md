# Metadata Preservation in Image Transformations

This document describes the metadata preservation behavior during image transformations in Omera's transform pipeline (`crates/omera-scan/src/transform.rs`).

## Overview

When transforming images (resizing, format conversion, quality adjustment), Omera can preserve embedded generation metadata according to the configured `TransformMetadataPolicy`. This ensures that AI generation parameters like prompts, model names, and settings are retained or stripped as needed.

## Metadata Policies

Three policies control metadata preservation:

### KeepSupported (Default)
Preserves all supported embedded generation metadata in the output file. For PNG files, this includes the `parameters` text chunk containing A1111-format generation metadata (prompt, negative prompt, steps, sampler, model, etc.).

### StripAi
Removes AI-specific generation metadata while potentially preserving other metadata:
- Removes: `prompt`, `negative_prompt`, `parameters`, `raw` fields
- The output file will not contain identifiable generation prompts or settings

### StripAll
Removes all metadata from the output file. The transformed image contains only pixel data with no embedded text chunks or EXIF tags.

## Supported Formats

### PNG ✓ Fully Supported
- **Preservation:** A1111 `parameters` text chunk is preserved in PNG-to-PNG transformations
- **Verification:** Metadata is verified by reopening the published derivative bytes
- **Round-trip:** PNG → resize → PNG preserves embedded generation metadata

### JPEG, WebP, AVIF — Not Yet Implemented
- **Current behavior:** Metadata is not preserved during transformations to or from these formats
- **Reason:** These formats require format-specific metadata embedding:
  - JPEG: EXIF `Software` tag or comment segments
  - WebP: Extended format chunks
  - AVIF: EXIF or XMP metadata boxes
- **Future work:** Format-specific metadata preservation can be added following the PNG implementation pattern

### Video (MP4, WebM) — Out of Scope
- Video transformations are not currently supported in the transform pipeline
- Video files can be imported without transformation

## Implementation Details

### Metadata Extraction
Source metadata is extracted at the start of `transform_file_staged`:
```rust
// Extract source metadata using detect_container + extract_metadata
let source_metadata = {
    let mut file = File::open(source_path)?;
    let mut header = [0u8; 32];
    let _ = file.read(&mut header);
    omera_metadata::detect_container(&header)
        .and_then(|c| omera_metadata::extract_metadata(c, source_path))
};
```

### Policy Application
Metadata is filtered based on the policy before encoding:
```rust
let metadata_to_write = match spec.metadata_policy {
    TransformMetadataPolicy::StripAll => None,
    TransformMetadataPolicy::StripAi => {
        source_metadata.as_ref().map(|m| {
            let mut stripped = m.clone();
            stripped.prompt = None;
            stripped.negative_prompt = None;
            stripped.parameters = None;
            stripped.raw = None;
            stripped
        })
    }
    TransformMetadataPolicy::KeepSupported => source_metadata.as_ref().cloned(),
};
```

### PNG Encoding with Metadata
The `encode_png_with_metadata` function writes metadata as PNG text chunks:
```rust
// Add metadata text chunks if present
if let Some(meta) = metadata {
    if let Some(ref params) = meta.parameters {
        encoder
            .add_text_chunk("parameters".into(), params.clone())
            .map_err(|e| TransformError::EncodeFailed(format!("Failed to add parameters chunk: {e}")))?;
    }
}
```

## Testing

Comprehensive test coverage in `crates/omera-scan/tests/metadata_preservation_test.rs`:
- `test_keep_supported_preserves_png_metadata`: Verifies PNG parameters chunk preservation
- `test_strip_ai_removes_generation_metadata`: Confirms AI metadata removal
- `test_strip_all_removes_all_metadata`: Ensures complete metadata stripping
- `test_jpeg_transform_documents_limitation`: Documents JPEG limitation

## Future Enhancements

### JPEG Metadata Preservation
- Embed generation parameters in EXIF `UserComment` or `ImageDescription` tags
- Use JPEG comment segments for A1111-compatible metadata
- Require `kamadak-exif` crate for EXIF writing

### WebP Metadata Preservation
- Use WebP extended format with EXIF or XMP chunks
- Maintain lossless encoding for metadata-bearing files

### AVIF Metadata Preservation
- Embed metadata in AVIF's EXIF or XMP boxes
- Leverage `libavif` or pure-Rust AVIF metadata writers

### Format Detection Optimization
- Cache container detection to avoid re-reading file headers
- Pass detected container through the transform pipeline

## Privacy and Security

The `StripAi` and `StripAll` policies are designed for privacy-sensitive workflows:
- **Export workflows:** Strip metadata before sharing images publicly
- **Batch transformations:** Remove identifying generation parameters from library copies
- **Vault imports:** Apply stripping policies at ingestion time

These policies ensure that prompts, model names, and other generation metadata do not leak when users share transformed images.

## Related Documentation

- [Image Transform Plan](IMAGE_TRANSFORM_PLAN.md) - Overall transformation architecture
- [Engineering Handoff](ENGINEERING_HANDOFF.md) - Task ownership and boundaries
- Issue [#237](https://github.com/BerryUIKI/Omera/issues/237) - KeepSupported transformation drops embedded generation metadata
