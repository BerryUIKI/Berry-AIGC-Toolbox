# Metadata Preservation in Image Transformations and Export

This document describes the metadata preservation behavior during image transformations and export operations in Omera.

## Overview

When transforming or exporting images (resizing, format conversion, quality adjustment), Omera can preserve embedded generation metadata according to the configured policy. This ensures that AI generation parameters like prompts, model names, and settings are retained or stripped as needed.

## Metadata Policies

### Export Privacy Modes

Export operations use `MetadataPrivacyMode`:

#### KeepAll
Preserves all supported embedded generation metadata in the output file. For PNG files, this includes the `parameters` text chunk containing A1111-format generation metadata (prompt, negative prompt, steps, sampler, model, etc.).

Metadata is extracted from source file bytes (not stale indexed data) to ensure accuracy during export.

#### StripPromptOnly
Removes prompt and negative_prompt fields from sidecars and showcase output, but does not currently embed stripped metadata into re-encoded image files.

#### StripAllAiMetadata
Removes all AI-specific generation metadata (prompt, negative_prompt, parameters, model, sampler, seed, steps, cfg_scale) from sidecars and showcase output.

#### StripAll
Removes all metadata from sidecars and showcase output.

### Transform Metadata Policies

Transformation operations use `TransformMetadataPolicy`:

#### KeepSupported (Default)
Preserves all supported embedded generation metadata in the output file. For PNG files, this includes the `parameters` text chunk containing A1111-format generation metadata.

#### StripAi
Removes AI-specific generation metadata while potentially preserving other metadata:
- Removes: `prompt`, `negative_prompt`, `parameters`, `raw` fields
- The output file will not contain identifiable generation prompts or settings

#### StripAll
Removes all metadata from the output file. The transformed image contains only pixel data with no embedded text chunks or EXIF tags.

## Supported Formats

### PNG ✓ Fully Supported (Transform and Export)
- **Preservation:** A1111 `parameters` text chunk is preserved in PNG-to-PNG transformations and PNG exports with KeepAll/KeepSupported
- **Verification:** Metadata is verified by reopening the published derivative bytes
- **Round-trip:** PNG → resize → PNG preserves embedded generation metadata
- **Export:** PNG exports with KeepAll and resizing extract metadata from source bytes and preserve the parameters chunk

### JPEG, WebP, AVIF — Not Yet Implemented
- **Current behavior:** Metadata is not preserved during transformations or exports to/from these formats
- **Reason:** These formats require format-specific metadata embedding:
  - JPEG: EXIF `Software` tag or comment segments
  - WebP: Extended format chunks
  - AVIF: EXIF or XMP metadata boxes
- **Future work:** Format-specific metadata preservation can be added following the PNG implementation pattern

### Video (MP4, WebM) — Out of Scope
- Video transformations are not currently supported in the transform or export pipelines
- Video files can be imported without transformation

## Implementation Details

### Export Metadata Extraction

Source metadata is extracted from file bytes at the start of `process_single_image` when needed:
```rust
// Extract source metadata from actual file bytes when needed for preservation
let source_metadata = if options.privacy == MetadataPrivacyMode::KeepAll
    && (options.format == ExportFormat::Png
        || (options.format == ExportFormat::Original && target_ext == "png"))
{
    let mut file_handle = File::open(src_path)?;
    let mut header = [0u8; 32];
    let _ = file_handle.read(&mut header);
    omera_metadata::detect_container(&header)
        .and_then(|c| omera_metadata::extract_metadata(c, src_path))
} else {
    None
};
```

### Transform Metadata Extraction

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

Both `transform.rs` and `export.rs` use PNG encoders that write metadata as text chunks:
```rust
// Add metadata text chunks if present
if let Some(meta) = metadata {
    if let Some(ref params) = meta.parameters {
        encoder
            .add_text_chunk("parameters".into(), params.clone())
            .map_err(|e| format!("Failed to add parameters chunk: {e}")))?;
    }
}
```

### Fast-Path Byte-Exact Preservation

Export operations preserve the original file byte-for-byte when:
- Format is `Original`
- Privacy mode is `KeepAll`
- No resizing is applied (`max_edge` is `None`)

This fast path ensures zero quality loss and exact metadata preservation.

## Testing

Comprehensive test coverage in:
- `crates/omera-scan/src/export.rs` tests
- `crates/omera-scan/tests/metadata_preservation_test.rs`

Export tests:
- `test_keepall_png_export_preserves_embedded_metadata_with_resize`: Verifies PNG parameters chunk preservation with resize
- `test_keepall_original_png_with_resize_preserves_metadata`: Confirms Original format PNG export preserves metadata
- `test_byte_exact_passthrough_with_keepall_no_resize`: Ensures byte-exact fast path
- `test_absent_metadata_export_does_not_fail`: Verifies graceful handling of files without metadata

Transform tests:
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

The `StripAi`, `StripPromptOnly`, `StripAllAiMetadata`, and `StripAll` policies are designed for privacy-sensitive workflows:
- **Export workflows:** Strip metadata before sharing images publicly
- **Batch transformations:** Remove identifying generation parameters from library copies
- **Vault imports:** Apply stripping policies at ingestion time

These policies ensure that prompts, model names, and other generation metadata do not leak when users share transformed or exported images.

**Important:** Privacy policies prevent re-introduction of stripped prompts into:
- Embedded image chunks (PNG text chunks, EXIF tags)
- Sidecar files (`.txt`, `.json`)
- Showcase HTML output

## Related Documentation

- [Image Transform Plan](IMAGE_TRANSFORM_PLAN.md) - Overall transformation architecture
- [Engineering Handoff](ENGINEERING_HANDOFF.md) - Task ownership and boundaries
- Issue [#237](https://github.com/BerryUIKI/Omera/issues/237) - KeepSupported transformation drops embedded generation metadata
- Issue [#272](https://github.com/BerryUIKI/Omera/issues/272) - KeepAll export strips embedded metadata during re-encoding
