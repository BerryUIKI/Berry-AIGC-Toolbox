//! Core domain types shared across Berry-AIGC-Toolbox crates.
//!
//! These types describe the *problem domain* — AI-generated image files and
//! their embedded metadata — independent of any storage or extraction concern.
//! Crates may depend on `berry-domain`, but it depends on nothing else.

mod extracted_metadata;
mod folder;
mod image_file;
mod metadata_format;

pub use extracted_metadata::ExtractedMetadata;
pub use folder::Folder;
pub use image_file::{Container, ImageFile};
pub use metadata_format::MetadataFormat;
