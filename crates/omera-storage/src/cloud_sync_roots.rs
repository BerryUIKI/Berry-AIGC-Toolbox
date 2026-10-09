//! Folder-linked cloud identity allocation. Does not access media or remotes.

use omera_domain::CloudSyncRootIdentity;
use rusqlite::{params, OptionalExtension};
use uuid::Uuid;

use crate::{Database, DatabaseError};

impl Database {
    /// Read the existing mapping without allocating identities during discovery.
    pub fn cloud_sync_root_identity(
        &self,
        folder_id: i64,
    ) -> Result<Option<CloudSyncRootIdentity>, DatabaseError> {
        Ok(self
            .connection()
            .query_row(
                "SELECT root_uuid, legacy_basename FROM cloud_sync_roots WHERE folder_id = ?1",
                [folder_id],
                |row| {
                    Ok(CloudSyncRootIdentity {
                        folder_id,
                        root_uuid: row.get(0)?,
                        legacy_basename: row.get(1)?,
                    })
                },
            )
            .optional()?)
    }

    /// Allocate once per folder registration. Existing identities are immutable;
    /// deleting a folder cascades its mapping, so reused numeric IDs are safe.
    pub fn ensure_cloud_sync_root(
        &self,
        folder_id: i64,
    ) -> Result<CloudSyncRootIdentity, DatabaseError> {
        if let Some(identity) = self.cloud_sync_root_identity(folder_id)? {
            return Ok(identity);
        }
        let path: String = self
            .connection()
            .query_row(
                "SELECT path FROM folders WHERE id = ?1",
                [folder_id],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(DatabaseError::FolderNotFound(folder_id))?;
        // Normalize separators even when reading a snapshot from another OS.
        let normalized = path.replace('\\', "/");
        let basename = normalized
            .trim_end_matches('/')
            .rsplit('/')
            .next()
            .filter(|name| !name.is_empty())
            .unwrap_or("library");
        self.connection().execute(
            "INSERT INTO cloud_sync_roots(folder_id, root_uuid, legacy_basename)
             VALUES (?1, ?2, ?3) ON CONFLICT(folder_id) DO NOTHING",
            params![folder_id, Uuid::new_v4().to_string(), basename],
        )?;
        // Concurrent allocation on a different connection must use the winner.
        self.cloud_sync_root_identity(folder_id)?
            .ok_or(DatabaseError::FolderNotFound(folder_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_basename_roots_receive_distinct_stable_uuids() {
        let db = Database::connect_in_memory().unwrap();
        let first = db.add_folder("/a/outputs").unwrap();
        let second = db.add_folder(r"E:\b\outputs\").unwrap();
        let a = db.ensure_cloud_sync_root(first.id).unwrap();
        let b = db.ensure_cloud_sync_root(second.id).unwrap();
        assert_eq!(a.legacy_basename, "outputs");
        assert_eq!(b.legacy_basename, "outputs");
        assert_ne!(a.root_uuid, b.root_uuid);
        assert!(Uuid::parse_str(&a.root_uuid).is_ok());
        assert!(Uuid::parse_str(&b.root_uuid).is_ok());
        assert_eq!(db.ensure_cloud_sync_root(first.id).unwrap(), a);
        assert_eq!(db.cloud_sync_root_identity(first.id).unwrap(), Some(a));
        assert_eq!(db.cloud_sync_root_identity(99).unwrap(), None);
    }

    #[test]
    fn identity_and_legacy_name_survive_rename_restart_and_backup() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("library.db");
        let backup = dir.path().join("backup.db");
        let identity;
        {
            let db = Database::connect(&path).unwrap();
            let folder = db.add_folder("/a/outputs").unwrap();
            identity = db.ensure_cloud_sync_root(folder.id).unwrap();
            db.connection()
                .execute(
                    "UPDATE folders SET path = '/new/renamed' WHERE id = ?1",
                    [folder.id],
                )
                .unwrap();
            assert_eq!(db.ensure_cloud_sync_root(folder.id).unwrap(), identity);
            db.backup_database(backup.to_str().unwrap()).unwrap();
        }
        let reopened = Database::connect(&path).unwrap();
        let restored = Database::connect(&backup).unwrap();
        assert_eq!(
            reopened.ensure_cloud_sync_root(identity.folder_id).unwrap(),
            identity
        );
        assert_eq!(
            restored.ensure_cloud_sync_root(identity.folder_id).unwrap(),
            identity
        );
        assert_eq!(identity.legacy_basename, "outputs");
    }

    #[test]
    fn numeric_folder_id_reuse_does_not_reuse_cloud_identity() {
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder("/a/outputs").unwrap();
        let old = db.ensure_cloud_sync_root(folder.id).unwrap();
        db.remove_folder(folder.id).unwrap();
        let new_folder = db.add_folder("/b/outputs").unwrap();
        assert_eq!(new_folder.id, folder.id); // SQLite may reuse the last ID.
        assert_ne!(
            db.ensure_cloud_sync_root(new_folder.id).unwrap().root_uuid,
            old.root_uuid
        );
    }

    #[test]
    fn missing_folder_and_rejected_allocation_leave_library_intact() {
        let db = Database::connect_in_memory().unwrap();
        assert!(matches!(
            db.ensure_cloud_sync_root(99),
            Err(DatabaseError::FolderNotFound(99))
        ));
        let folder = db.add_folder("/a/outputs").unwrap();
        db.connection()
            .execute_batch(
                "CREATE TRIGGER reject_cloud_identity BEFORE INSERT ON cloud_sync_roots
             BEGIN SELECT RAISE(ABORT, 'fixture rejection'); END;",
            )
            .unwrap();
        assert!(db.ensure_cloud_sync_root(folder.id).is_err());
        assert_eq!(db.list_folders().unwrap(), vec![folder]);
        let count: i64 = db
            .connection()
            .query_row("SELECT count(*) FROM cloud_sync_roots", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(count, 0);
    }
}
