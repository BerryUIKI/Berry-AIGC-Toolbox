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

    /// A stable identifier used for database storage and display.
    pub const fn id(self) -> &'static str {
        self.extension()
    }

    /// Look up a container by its stable [`id`](Self::id).
    pub fn from_id(id: &str) -> Option<Container> {
        match id {
            "png" => Some(Self::Png),
            "jpg" => Some(Self::Jpeg),
            "webp" => Some(Self::WebP),
            "mp4" => Some(Self::Mp4),
            "txt" => Some(Self::Txt),
            _ => None,
        }
    }
}

/// A single media file discovered by a scan and persisted to the `files` row.
///
/// `id` is `None` for a file that has not been inserted yet; the scanner
/// builds these from disk metadata and `Database::upsert_file` fills in the id.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImageFile {
    /// Database row id, `None` until the file is persisted.
    pub id: Option<i64>,
    /// Id of the [`Folder`](crate::Folder) this file was scanned from.
    pub folder_id: i64,
    /// Absolute path of the file on disk.
    pub path: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Filesystem modification time as unix seconds; the incremental-scan
    /// cache compares (size, mtime) to skip unchanged files.
    pub modified_at: i64,
    /// Detected container format.
    pub container: Container,
    /// Extracted generation metadata, `None` until extraction runs.
    pub metadata: Option<crate::ExtractedMetadata>,
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

    #[test]
    fn container_id_roundtrips() {
        for container in [
            Container::Png,
            Container::Jpeg,
            Container::WebP,
            Container::Mp4,
            Container::Txt,
        ] {
            assert_eq!(Container::from_id(container.id()), Some(container));
        }
        assert_eq!(Container::from_id("bmp"), None);
    }

    #[test]
    fn ids_are_also_extensions() {
        assert_eq!(Container::Jpeg.id(), "jpg");
        assert_eq!(Container::Jpeg.extension(), "jpg");
    }
}
