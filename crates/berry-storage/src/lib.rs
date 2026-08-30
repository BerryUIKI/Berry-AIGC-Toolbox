//! SQLite persistence with schema versioning for Berry-AIGC-Toolbox.
//!
//! Owns the SQLite connection and applies an ordered list of embedded
//! migrations tracked by SQLite's `PRAGMA user_version`. All schema changes
//! go through `migrations`, never through ad-hoc DDL.

mod db;
mod migrations;

pub use db::{Database, DatabaseError};
