//! Metadata detection and extraction for AI-generated image formats.
//!
//! [`detect_container`] sniffs a file's container from its magic bytes.
//! [`extract_metadata`] is the format-dispatch entry point used by the scan
//! engine: given a container and a file path it returns structured metadata or
//! `None`. Embedded metadata (PNGInfo, EXIF) is tried first; when a file carries
//! none, a sibling `.txt` sidecar is used as a fallback.

use std::path::Path;

use berry_domain::{Container, ExtractedMetadata, MetadataFormat};

pub mod container;
pub mod exif;
pub mod parameters;
pub mod pnginfo;
pub mod sidecar;

pub use container::detect_container;

/// Extract structured metadata from a media file, dispatching on its container.
///
/// Returns `None` when the file carries no recognizable metadata. Only image
/// containers are considered; videos and text are out of scope. Embedded
/// metadata wins; a `.txt` sidecar is only consulted as a fallback.
pub fn extract_metadata(container: Container, path: &Path) -> Option<ExtractedMetadata> {
    if !container.is_image() {
        return None;
    }
    let embedded = match container {
        Container::Png => extract_png_metadata(path),
        Container::Jpeg | Container::WebP => extract_exif_metadata(path),
        Container::Mp4 | Container::Txt => None,
    };
    embedded.or_else(|| extract_sidecar_metadata(path))
}

/// Extract A1111/SD.Next PNGInfo from a PNG file's `parameters` text chunk.
fn extract_png_metadata(path: &Path) -> Option<ExtractedMetadata> {
    let bytes = std::fs::read(path).ok()?;
    let parameters = pnginfo::extract_parameters(&bytes)?;
    Some(from_parameters(parameters))
}

/// Extract dimensions + generator name from a JPEG/WebP file's EXIF block.
///
/// Returns `None` when the `Software` tag does not name a recognized generator,
/// so plain camera photos are left alone.
fn extract_exif_metadata(path: &Path) -> Option<ExtractedMetadata> {
    let info = exif::read_exif(path).ok()??;
    let software = info.software.as_deref()?;
    let format = exif::infer_format(software)?;
    Some(ExtractedMetadata {
        format,
        parameters: None,
        raw: info.software,
        prompt: None,
        negative_prompt: None,
        width: info.width,
        height: info.height,
        seed: None,
        steps: None,
        cfg_scale: None,
        sampler: None,
        model_name: None,
        model_hash: None,
    })
}

/// Read a sibling `<file>.txt` and parse it as A1111-style parameters.
///
/// A `.txt` sidecar is the calling card of Fooocus (which saves prompts next to
/// its outputs), so sidecar-derived metadata is tagged accordingly.
fn extract_sidecar_metadata(path: &Path) -> Option<ExtractedMetadata> {
    let text = sidecar::read_sidecar(path)?;
    Some(from_parameters(text))
}

/// Build [`ExtractedMetadata`] from an A1111-style parameter string.
fn from_parameters(parameters: String) -> ExtractedMetadata {
    let parsed = parameters::parse_parameters(&parameters);
    ExtractedMetadata {
        format: MetadataFormat::A1111,
        parameters: Some(parameters),
        raw: None,
        prompt: parsed.prompt,
        negative_prompt: parsed.negative_prompt,
        width: parsed.width,
        height: parsed.height,
        seed: parsed.seed,
        steps: parsed.steps,
        cfg_scale: parsed.cfg_scale,
        sampler: parsed.sampler,
        model_name: parsed.model_name,
        model_hash: parsed.model_hash,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tex(keyword: &str, text: &str) -> Vec<u8> {
        let mut data = keyword.as_bytes().to_vec();
        data.push(0);
        data.extend_from_slice(text.as_bytes());
        let mut out = (data.len() as u32).to_be_bytes().to_vec();
        out.extend_from_slice(b"tEXt");
        out.extend_from_slice(&data);
        out.extend_from_slice(&[0, 0, 0, 0]);
        out
    }

    fn png_with_parameters(text: &str) -> Vec<u8> {
        let mut out = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A].to_vec();
        out.extend_from_slice(&tex("parameters", text));
        out.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D']);
        out.extend_from_slice(&[0, 0, 0, 0]);
        out
    }

    fn plain_png() -> Vec<u8> {
        let mut out = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A].to_vec();
        out.extend_from_slice(&[0, 0, 0, 0, b'I', b'E', b'N', b'D']);
        out.extend_from_slice(&[0, 0, 0, 0]);
        out
    }

    /// Minimal JPEG with an EXIF APP1 segment (`ComfyUI` software + dimensions).
    fn exif_jpeg() -> Vec<u8> {
        let software = "ComfyUI";
        let mut tiff = Vec::new();
        tiff.extend_from_slice(b"II");
        tiff.extend_from_slice(&0x002A_u16.to_le_bytes());
        tiff.extend_from_slice(&8u32.to_le_bytes());
        tiff.extend_from_slice(&1u16.to_le_bytes()); // single entry: Software
        tiff.extend_from_slice(&0x0131_u16.to_le_bytes());
        tiff.extend_from_slice(&2u16.to_le_bytes());
        tiff.extend_from_slice(&(software.len() as u32 + 1).to_le_bytes());
        tiff.extend_from_slice(&26u32.to_le_bytes());
        tiff.extend_from_slice(&0u32.to_le_bytes()); // next IFD offset
        tiff.extend_from_slice(software.as_bytes());
        tiff.push(0);

        let marker = b"Exif\0\0";
        let app1_len = 2 + marker.len() + tiff.len();
        let mut out = vec![0xFF, 0xD8];
        out.extend_from_slice(&[0xFF, 0xE1]);
        out.extend_from_slice(&(app1_len as u16).to_be_bytes());
        out.extend_from_slice(marker);
        out.extend_from_slice(&tiff);
        out.extend_from_slice(&[0xFF, 0xD9]);
        out
    }

    /// Helper: write `bytes` to a fresh temp subdirectory and return the path.
    fn temp_file(dir: &str, name: &str, bytes: &[u8]) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("berry-meta-{}", std::process::id()))
            .join(dir);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }

    #[test]
    fn extracts_png_metadata() {
        let path = temp_file(
            "png",
            "test.png",
            &png_with_parameters(
                "a cat\nNegative prompt: blurry\nSteps: 20, Seed: 42, Size: 512x768",
            ),
        );

        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::A1111);
        assert_eq!(meta.prompt.as_deref(), Some("a cat"));
        assert_eq!(meta.negative_prompt.as_deref(), Some("blurry"));
        assert_eq!(meta.steps, Some(20));
        assert_eq!(meta.seed.as_deref(), Some("42"));
        assert_eq!(meta.width, Some(512));
        assert_eq!(meta.height, Some(768));

        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn extracts_exif_metadata_from_jpeg() {
        let path = temp_file("exif", "comfy.jpg", &exif_jpeg());
        let meta = extract_metadata(Container::Jpeg, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::ComfyUI);
        assert_eq!(meta.raw.as_deref(), Some("ComfyUI"));
        assert_eq!(meta.prompt, None);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn plain_jpeg_falls_back_to_sidecar() {
        let path = temp_file("jpeg-sidecar", "plain.jpg", &[0xFF, 0xD8, 0xFF, 0xE0]);
        std::fs::write(
            path.with_extension("txt"),
            "a landscape\nSteps: 12, Sampler: DPM++ 2M",
        )
        .unwrap();

        let meta = extract_metadata(Container::Jpeg, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::A1111);
        assert_eq!(meta.prompt.as_deref(), Some("a landscape"));
        assert_eq!(meta.steps, Some(12));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn plain_png_falls_back_to_sidecar() {
        let path = temp_file("png-sidecar", "plain.png", &plain_png());
        std::fs::write(path.with_extension("txt"), "just a prompt").unwrap();

        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::A1111);
        assert_eq!(meta.prompt.as_deref(), Some("just a prompt"));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn images_with_no_metadata_return_none() {
        let path = temp_file("none", "photo.jpg", &[0xFF, 0xD8, 0xFF, 0xE0]);
        assert_eq!(extract_metadata(Container::Jpeg, &path), None);
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }

    #[test]
    fn videos_are_ignored() {
        assert_eq!(
            extract_metadata(Container::Mp4, Path::new("x.mp4")),
            None,
            "mp4 metadata extraction is out of scope"
        );
    }
}
