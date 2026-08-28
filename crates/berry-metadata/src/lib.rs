//! Metadata detection and extraction for AI-generated image formats.
//!
//! M1 establishes the module skeleton and the magic-byte container detector
//! that M2's scanner relies on. The per-format parsers (`pnginfo`, `exif`,
//! `sidecar`) are declared here and filled in during M2.

pub mod container;
pub mod exif;
pub mod pnginfo;
pub mod sidecar;

pub use container::detect_container;
