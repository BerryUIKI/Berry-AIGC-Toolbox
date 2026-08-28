//! The media container of a scanned file.

use serde::{Deserialize, Serialize};

/// The file container format of a scanned media file.
///
/// Detection by magic bytes lives in `berry-metadata`; this type only
/// describes the possible outcomes so the rest of the app can branch on it
/// (e.g. "only PNG/JPEG/WebP carry embedded PNGInfo/EXIF metadata").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Container {
    /// Portable Network Graphics (`.png`).
    Png,
    /// JPEG / JFIF (`.jpg`, `.jpeg`).
    Jpeg,
    /// WebP (`.webp`).
    WebP,
    /// MP4 (`.mp4`).
    Mp4,
    /// Plain text (`.txt`) sidecar metadata file.
    Txt,
}

impl Container {
    /// Whether this container holds an image (rather than a video or text).
    pub const fn is_image(self) -> bool {
        matches!(self, Self::Png | Self::Jpeg | Self::WebP)
    }

    /// Whether this container holds a video.
    pub const fn is_video(self) -> bool {
        matches!(self, Self::Mp4)
    }

    /// A canonical file extension for the container, without the leading dot.
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Jpeg => "jpg",
            Self::WebP => "webp",
            Self::Mp4 => "mp4",
            Self::Txt => "txt",
        }
    }
}

/// A single media file discovered by a scan.
///
/// M1 models the minimum fields the index will need; the schema grows as
/// scanning and metadata extraction land in M2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageFile {
    /// Absolute path of the file on disk.
    pub path: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Detected container format.
    pub container: Container,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn images_are_not_videos() {
        for container in [Container::Png, Container::Jpeg, Container::WebP] {
            assert!(container.is_image());
            assert!(!container.is_video());
        }
    }

    #[test]
    fn mp4_is_a_video() {
        assert!(Container::Mp4.is_video());
        assert!(!Container::Mp4.is_image());
    }

    #[test]
    fn serde_roundtrip() {
        let json = serde_json::to_string(&Container::WebP).unwrap();
        let back: Container = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Container::WebP);
    }
}
