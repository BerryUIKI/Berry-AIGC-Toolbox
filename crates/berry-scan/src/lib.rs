//! Folder scanning and indexing orchestration.
//!
//! `berry-scan` ties the core crates together into a single operation the app
//! shell can call: walk a folder recursively, detect each media file's
//! container, extract metadata (via a pluggable extractor from
//! `berry-metadata`), persist rows through `berry-storage`, and drop rows for
//! files that disappeared from disk. The Tauri shell only wires this up.

pub mod scanner;

pub use scanner::{ScanError, ScanProgress, ScanStats, Scanner};
