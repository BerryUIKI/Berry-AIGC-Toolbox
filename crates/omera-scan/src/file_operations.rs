//! Recoverable file operations. Destinations are never overwritten.
use omera_storage::Database;
use std::{fs, io::Write, path::Path};

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Copy,
    Move,
    Trash,
}

pub fn execute(
    db_path: &Path,
    paths: &[String],
    destination: Option<i64>,
    operation: Operation,
) -> Result<usize, String> {
    let db = Database::connect(db_path).map_err(|e| e.to_string())?;
    let all_folders = db.list_folders().map_err(|e| e.to_string())?;
    let target = destination
        .map(|id| {
            all_folders
                .iter()
                .find(|folder| folder.id == id)
                .cloned()
                .ok_or_else(|| "Destination folder not found".to_string())
        })
        .transpose()?;

    if let Some(ref target_folder) = target {
        if target_folder.folder_type == "link" {
            return Err("Destination folder cannot be an external linked folder; linked folders are read-only".into());
        }
    }

    let journal_path = db_path.with_extension("file-operations.jsonl");
    let mut journal = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&journal_path)
        .map_err(|e| e.to_string())?;
    let mut completed = 0;
    let mut errors = Vec::new();
    for path in paths {
        let result = (|| -> Result<(), String> {
            let original = db
                .get_file_by_path(path)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Source is not indexed".to_string())?;

            let source_folder = all_folders.iter().find(|f| f.id == original.folder_id);
            if matches!(operation, Operation::Move)
                && source_folder.is_some_and(|f| f.folder_type == "link")
            {
                return Err(format!(
                    "Cannot move '{}': source is in an external linked folder (linked folders are read-only)",
                    path
                ));
            }
            let source = Path::new(path);
            let mut sources = vec![source.to_path_buf()];
            for extension in ["txt", "json"] {
                let sidecar = source.with_extension(extension);
                if sidecar != source && sidecar.is_file() {
                    sources.push(sidecar);
                }
            }
            let destinations = if let Some(folder) = &target {
                sources
                    .iter()
                    .map(|source| {
                        Path::new(&folder.path).join(source.file_name().unwrap_or_default())
                    })
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };
            if destinations.iter().any(|path| path.exists()) {
                return Err("Destination already exists; original preserved".into());
            }
            // Persist the operation plan before any filesystem mutation for recovery.
            writeln!(journal, "{}", serde_json::json!({"state":"started", "operation":format!("{operation:?}"), "sources":sources,"destinations":destinations}))
                .and_then(|_| journal.sync_all()).map_err(|e| e.to_string())?;
            if matches!(operation, Operation::Trash) {
                // No permanent-delete fallback, including sidecars.
                trash::delete_all(&sources).map_err(|e| e.to_string())?;
                db.delete_file_by_path(path).map_err(|e| e.to_string())?;
            } else {
                let mut published = Vec::new();
                for (source, destination) in sources.iter().zip(&destinations) {
                    if let Err(error) = copy_new(source, destination) {
                        for path in published {
                            let _ = fs::remove_file(path);
                        }
                        return Err(error.to_string());
                    }
                    published.push(destination.clone());
                }
                let mut copied = original.clone();
                copied.id = None;
                copied.path = destinations[0].to_string_lossy().into_owned();
                copied.folder_id = destination.ok_or("Missing destination")?;
                copied.stack_id = None;
                copied.stack_order = 0;
                // Persist destination before removing any source. Failed writes leave originals safe.
                let persisted = if matches!(operation, Operation::Copy) {
                    db.upsert_file(&copied).map(|_| ())
                } else {
                    db.move_file_record(path, &copied.path, copied.folder_id)
                        .map(|_| ())
                };
                if let Err(error) = persisted {
                    for path in published {
                        let _ = fs::remove_file(path);
                    }
                    return Err(error.to_string());
                }
                if matches!(operation, Operation::Move) {
                    for source in &sources {
                        fs::remove_file(source).map_err(|e| {
                            format!(
                                "Destination saved at {}; source cleanup failed: {e}",
                                copied.path
                            )
                        })?;
                    }
                }
            }
            writeln!(
                journal,
                "{}",
                serde_json::json!({"state":"completed", "source":path})
            )
            .and_then(|_| journal.sync_all())
            .map_err(|e| e.to_string())?;
            Ok(())
        })();
        match result {
            Ok(()) => completed += 1,
            Err(error) => errors.push(format!("{path}: {error}")),
        }
    }
    if errors.is_empty() {
        Ok(completed)
    } else {
        Err(format!(
            "{completed}/{} completed. Recovery journal: {}\n{}",
            paths.len(),
            journal_path.display(),
            errors.join("\n")
        ))
    }
}

fn copy_new(source: &Path, destination: &Path) -> std::io::Result<()> {
    let mut temporary = tempfile::NamedTempFile::new_in(
        destination
            .parent()
            .ok_or_else(|| std::io::Error::other("Missing destination directory"))?,
    )?;
    std::io::copy(&mut fs::File::open(source)?, temporary.as_file_mut())?;
    temporary.as_file().sync_all()?;
    temporary
        .persist_noclobber(destination)
        .map_err(|e| e.error)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn copy_never_overwrites_an_existing_destination() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source");
        let destination = dir.path().join("destination");
        fs::write(&source, b"new").unwrap();
        fs::write(&destination, b"original").unwrap();
        assert!(copy_new(&source, &destination).is_err());
        assert_eq!(fs::read(destination).unwrap(), b"original");
        assert_eq!(fs::read(source).unwrap(), b"new");
    }

    #[test]
    fn cannot_move_from_linked_folder() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("omera.db");
        let db = Database::connect(&db_path).unwrap();

        let link_dir = dir.path().join("linked");
        let managed_dir = dir.path().join("managed");
        fs::create_dir_all(&link_dir).unwrap();
        fs::create_dir_all(&managed_dir).unwrap();

        let link_folder = db.add_folder(&link_dir.to_string_lossy()).unwrap();
        let managed_folder = db
            .add_folder_with_mode(
                &managed_dir.to_string_lossy(),
                "managed",
                None,
                None,
                None,
                false,
            )
            .unwrap();

        let file_path = link_dir.join("photo.png");
        fs::write(&file_path, b"dummy png content").unwrap();

        let image_file = omera_domain::ImageFile {
            id: None,
            folder_id: link_folder.id,
            path: file_path.to_string_lossy().to_string(),
            size_bytes: 17,
            modified_at: 1000,
            container: omera_domain::Container::Png,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        db.upsert_file(&image_file).unwrap();

        // Attempting to move from linked folder must fail
        let res = execute(
            &db_path,
            &[file_path.to_string_lossy().to_string()],
            Some(managed_folder.id),
            Operation::Move,
        );
        assert!(res.is_err());
        let err_msg = res.unwrap_err();
        assert!(err_msg.contains("linked folders are read-only"));

        // File must still exist on disk untouched
        assert!(file_path.exists());
        assert_eq!(fs::read(&file_path).unwrap(), b"dummy png content");
    }

    #[test]
    fn cannot_copy_or_move_into_linked_folder() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("omera.db");
        let db = Database::connect(&db_path).unwrap();

        let link_dir = dir.path().join("linked");
        let managed_dir = dir.path().join("managed");
        fs::create_dir_all(&link_dir).unwrap();
        fs::create_dir_all(&managed_dir).unwrap();

        let link_folder = db.add_folder(&link_dir.to_string_lossy()).unwrap();
        let managed_folder = db
            .add_folder_with_mode(
                &managed_dir.to_string_lossy(),
                "managed",
                None,
                None,
                None,
                false,
            )
            .unwrap();

        let file_path = managed_dir.join("photo.png");
        fs::write(&file_path, b"vault content").unwrap();

        let image_file = omera_domain::ImageFile {
            id: None,
            folder_id: managed_folder.id,
            path: file_path.to_string_lossy().to_string(),
            size_bytes: 13,
            modified_at: 1000,
            container: omera_domain::Container::Png,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        db.upsert_file(&image_file).unwrap();

        // Attempting to copy into linked folder must fail
        let res = execute(
            &db_path,
            &[file_path.to_string_lossy().to_string()],
            Some(link_folder.id),
            Operation::Copy,
        );
        assert!(res.is_err());
        assert!(res
            .unwrap_err()
            .contains("Destination folder cannot be an external linked folder"));

        // Attempting to move into linked folder must fail
        let res_move = execute(
            &db_path,
            &[file_path.to_string_lossy().to_string()],
            Some(link_folder.id),
            Operation::Move,
        );
        assert!(res_move.is_err());
        assert!(res_move
            .unwrap_err()
            .contains("Destination folder cannot be an external linked folder"));
    }

    #[test]
    fn copy_from_linked_to_managed_folder_preserves_original() {
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("omera.db");
        let db = Database::connect(&db_path).unwrap();

        let link_dir = dir.path().join("linked");
        let managed_dir = dir.path().join("managed");
        fs::create_dir_all(&link_dir).unwrap();
        fs::create_dir_all(&managed_dir).unwrap();

        let link_folder = db.add_folder(&link_dir.to_string_lossy()).unwrap();
        let managed_folder = db
            .add_folder_with_mode(
                &managed_dir.to_string_lossy(),
                "managed",
                None,
                None,
                None,
                false,
            )
            .unwrap();

        let file_path = link_dir.join("photo.png");
        fs::write(&file_path, b"original content").unwrap();

        let image_file = omera_domain::ImageFile {
            id: None,
            folder_id: link_folder.id,
            path: file_path.to_string_lossy().to_string(),
            size_bytes: 16,
            modified_at: 1000,
            container: omera_domain::Container::Png,
            metadata: None,
            rating: None,
            aesthetic_score: None,
            is_favorite: false,
            is_nsfw: false,
            stack_id: None,
            stack_order: 0,
        };
        db.upsert_file(&image_file).unwrap();

        // Copying from linked folder to managed folder must succeed
        let res = execute(
            &db_path,
            &[file_path.to_string_lossy().to_string()],
            Some(managed_folder.id),
            Operation::Copy,
        );
        assert_eq!(res.unwrap(), 1);

        // Original file must remain safe and intact
        assert!(file_path.exists());
        assert_eq!(fs::read(&file_path).unwrap(), b"original content");

        // Destination file was created
        let dest_file = managed_dir.join("photo.png");
        assert!(dest_file.exists());
        assert_eq!(fs::read(&dest_file).unwrap(), b"original content");
    }
}
