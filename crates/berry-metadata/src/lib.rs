//! Metadata detection and extraction for AI-generated image formats.
//!
//! [`detect_container`] sniffs a file's container from its magic bytes.
//! [`extract_metadata`] is the format-dispatch entry point used by the scan
//! engine: given a container and a file path it returns structured metadata or
//! `None`. PNGInfo (A1111/SD.Next) is wired up; EXIF and `.txt` sidecars are
//! added in later M2 steps.

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
/// containers are considered; videos and text are out of scope.
pub fn extract_metadata(container: Container, path: &Path) -> Option<ExtractedMetadata> {
    if !container.is_image() {
        return None;
    }
    match container {
        Container::Png => extract_png_metadata(path),
        // JPEG/WebP EXIF arrives in M2-5.
        Container::Jpeg | Container::WebP => None,
        Container::Mp4 | Container::Txt => None,
    }
}

/// Extract A1111/SD.Next PNGInfo from a PNG file's `parameters` text chunk.
fn extract_png_metadata(path: &Path) -> Option<ExtractedMetadata> {
    let bytes = std::fs::read(path).ok()?;
    let parameters = pnginfo::extract_parameters(&bytes)?;
    let parsed = parameters::parse_parameters(&parameters);
    Some(ExtractedMetadata {
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
    })
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

    #[test]
    fn extracts_png_metadata() {
        let dir = std::env::temp_dir().join(format!("berry-meta-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.png");
        std::fs::write(
            &path,
            png_with_parameters(
                "a cat\nNegative prompt: blurry\nSteps: 20, Seed: 42, Size: 512x768",
            ),
        )
        .unwrap();

        let meta = extract_metadata(Container::Png, &path).expect("extracted");
        assert_eq!(meta.format, MetadataFormat::A1111);
        assert_eq!(meta.prompt.as_deref(), Some("a cat"));
        assert_eq!(meta.negative_prompt.as_deref(), Some("blurry"));
        assert_eq!(meta.steps, Some(20));
        assert_eq!(meta.seed.as_deref(), Some("42"));
        assert_eq!(meta.width, Some(512));
        assert_eq!(meta.height, Some(768));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn non_png_images_return_none() {
        let dir = std::env::temp_dir().join(format!("berry-meta-none-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("photo.jpg");
        std::fs::write(&path, [0xFF, 0xD8, 0xFF, 0xE0]).unwrap();
        assert_eq!(extract_metadata(Container::Jpeg, &path), None);
        std::fs::remove_dir_all(&dir).unwrap();
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
