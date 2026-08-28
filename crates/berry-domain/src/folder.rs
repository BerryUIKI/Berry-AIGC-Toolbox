//! A user-added folder that the app indexes.

use serde::{Deserialize, Serialize};

/// A folder the user has asked the app to index.
///
/// Stored in the `folders` table (migration v2). Its scanned files live in
/// `files` and are cascade-deleted when the folder is removed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Folder {
    /// Database row id.
    pub id: i64,
    /// Absolute path of the folder.
    pub path: String,
    /// When the folder was added, as an ISO-8601 UTC timestamp.
    pub added_at: String,
}
