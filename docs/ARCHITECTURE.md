# Architecture

Berry-AIGC-Toolbox is a **Tauri 2 desktop app**: a Vue 3 + TypeScript frontend
running in the system WebView, a Rust core that owns all domain logic and
persistence, and SQLite as the metadata store.

```
┌──────────────────────────────┐
│  Frontend  (Vue 3 + TS)      │   src/            — UI shell
│  src/App.vue, src/main.ts    │
└──────────────┬───────────────┘
               │  Tauri IPC (@tauri-apps/api, #[tauri::command])
┌──────────────▼───────────────┐
│  App crate  (src-tauri/)     │   berry-aigc-toolbox — thin shell: wires
│  lib.rs, commands.rs         │   crates together, exposes IPC commands
└──────────────┬───────────────┘
               │
┌──────────────▼───────────────┐
│  berry-scan                  │   Scanner: walk folder → detect container
│  Scanner, ScanStats          │   → extract metadata → persist rows
└───┬──────────────┬───────────┘
    │              │
┌───▼────────────┐ │ ┌───────────────────────────┐
│  berry-domain  │ │ │  berry-metadata           │
│  shared types  │◀┘ │  format detection & parse │
│  (ImageFile,   │    │  (PNGInfo/EXIF/sidecar)  │
│   MetadataFormat)   └───────────────────────────┘
└───┬────────────┘
    │
┌───▼────────────┐
│  berry-storage │   SQLite persistence: schema
│  Database      │   versioning via PRAGMA user_version
└────────────────┘
```

## Crate layout

A single Cargo workspace at the repository root (`Cargo.toml`). Run `cargo test`,
`cargo clippy`, and `cargo fmt` from the root to cover every crate at once.

| Crate | Responsibility | Depends on |
|---|---|---|
| `berry-aigc-toolbox` (`src-tauri/`) | Tauri app shell: window setup, IPC commands, application state. **No business logic** — commands are thin adapters over the core crates. | berry-storage, berry-scan |
| `berry-domain` | Pure domain types shared across crates: `ImageFile`, `Folder`, `Container`, `ExtractedMetadata`, `MetadataFormat`. Depends on nothing in this repo. | serde |
| `berry-metadata` | Detecting and parsing generation metadata. `detect_container` sniffs a file's container from its magic bytes; `extract_metadata` dispatches on it — PNGInfo (`parameters` text chunks) for PNG, EXIF (`Software` tag + dimensions) for JPEG/WebP, with a sibling `.txt` sidecar as a fallback for both. | berry-domain, kamadak-exif |
| `berry-scan` | Folder scanning orchestration: walks a directory, detects containers, extracts metadata, upserts rows in batches, and cleans up orphan rows. Opens its own DB connection per scan so it never blocks the shell's. | berry-domain, berry-metadata, berry-storage, walkdir |
| `berry-storage` | SQLite connection + schema versioning. All schema changes go through the ordered `MIGRATIONS` list; there is no ad-hoc DDL. | berry-domain, rusqlite |

## Data flow

1. The frontend calls a `#[tauri::command]` (e.g. `get_app_info`) via
   `invoke()`.
2. The command locks shared state, calls into a core crate, and returns a
   serde-serializable value (or a `Result` mapped to a `String` error).
3. Long-lived resources (the SQLite `Database`) live in Tauri-managed state
   (`AppState` in `src-tauri/src/lib.rs`), opened once during `.setup()` in the
   OS app-data directory.

## Scanning

A scan (`berry-scan::Scanner`) is a single unit of work the app shell can call
from a `#[tauri::command]`:

1. Recursively walk the folder (`walkdir`), skipping hidden directories, and
   keep files with a supported media extension (`.png/.jpg/.jpeg/.webp/.mp4`).
2. For each file, compare `(size, modified_at)` against the stored row — if
   unchanged (and already extracted when an extractor is installed) it is
   skipped as part of the **incremental scan**.
3. Otherwise detect the container from magic bytes (falling back to the
   extension), run the metadata extractor, and upsert the row.
4. Upserts are batched (one transaction per 256 files); progress is reported
   through an `on_progress` callback that the shell forwards as Tauri events.
5. Finally, rows for files that disappeared from disk are deleted
   (`delete_files_not_in`).

The scanner opens its **own** SQLite connection to the same database file per
scan (WAL allows concurrent readers), so a long scan does not block the shell's
connection used by read commands.

### Rebuild metadata

`rebuild_metadata` runs the same scanner with extraction forced on: the
incremental cache is bypassed, so every file is re-read and re-extracted even
when its `(size, mtime)` is unchanged. Use it after a metadata-parser update so
files indexed under an older extractor pick up the new fields.

## Schema Versioning & Migrations

`berry-storage` tracks schema versions with SQLite's `PRAGMA user_version`.
`crates/berry-storage/src/migrations.rs` holds an ordered `MIGRATIONS: &[&str]`;
`Database::migrate()` applies each pending migration inside a transaction and
bumps `user_version` by one.

**Migration History:**
- **v1**: Initial tables (`folders`, `files`, `meta`).
- **v2**: Albums and relationships (`albums`, `album_files`).
- **v3**: Tagging system (`tags`, `file_tags`).
- **v4**: Favorites and NSFW classification flags (`is_favorite`, `is_nsfw` indexed columns in `files`).
- **v5**: Model cache & hash lookup table (`model_cache` with unique indexes on `hash` and `sha256`).

**Rules:**
- Never edit, reorder, or delete an applied migration — deployed databases
  depend on the sequence. Add a new entry to the end of `MIGRATIONS` instead.
- Every schema change ships with a migration, never ad-hoc `CREATE`/`ALTER`
  statements.

## Multi-Format Metadata Parsers

Metadata parsing in `berry-metadata` supports diverse generator pipelines through modular parsers:
- **AUTOMATIC1111 / SDNext (`parameters.rs`, `pnginfo.rs`)**: Standard A1111 parameter block parser extracting positive/negative prompts, samplers, seeds, steps, CFG, and hashes.
- **ComfyUI (`comfyui.rs`)**: JSON node graph decoder navigating prompt and workflow dictionaries to identify `KSampler`, `CLIPTextEncode`, `CheckpointLoaderSimple`, and `EmptyLatentImage` nodes.
- **NovelAI (`novelai.rs`)**: Comment JSON metadata extractor reading `prompt`, `uc` (undesired content), `steps`, `scale`, `sampler`, `seed`, and native resolution.
- **InvokeAI (`invokeai.rs`)**: Decoder for `sd-metadata` and `invokeai_metadata` JSON blocks.
- **Fooocus (`fooocus.rs`)**: Text parameter parser mapping `Prompt:`, `Negative:`, `Sampler:`, and `Model:` lines.
- **EasyDiffusion & Stable Swarm (`easydiffusion.rs`)**: Structured JSON parameters and metadata extractor.

## Organization & Curation

- **Albums (`AlbumModal.vue`)**: Custom collections with drag-and-drop support, sidebar counts, and multi-file assignment.
- **Tags (`TagModal.vue`)**: Color-coded taxonomy chips allowing categorization, filter queries (`tag:anime`), and batch tagging.
- **Favorites & NSFW**: Flagged in SQLite (`files.is_favorite`, `files.is_nsfw`) with quick-toggle hotkeys and privacy blur overlays.
- **Prompt Insights (`PromptStatsModal.vue`)**: Aggregates positive/negative keyword frequencies, average star ratings, top checkpoint models, and top samplers.

## Checkpoint Models & Hash Cache

- **Catalog & Resolver (`ModelManagerModal.vue`)**: Aggregates distinct models referenced across indexed metadata alongside image counts.
- **Hash Cache Integration**: Imports AUTOMATIC1111 `cache.json` dictionaries to resolve short model hashes (e.g. `e4a30e46`) and full SHA256 hashes back to human-readable checkpoint filenames and titles.

## File Operations & Drag-and-Drop

- **Cross-Platform File Management**: Native file moving (`move_files`), copying (`copy_files`), safe system trash deletion (`trash_files` via `trash` crate), and system file manager reveal (`reveal_in_file_manager`). Automatically manages associated sibling sidecars (`.txt`, `.json`).
- **Interactive Drag-and-Drop**: Native HTML5 Drag and Drop from `VirtualGrid.vue` onto sidebar `FolderList.vue` rows to move files or assign albums/tags.
- **Batch Action Toolbar (`BatchActionBar.vue`)**: Floating batch bar supporting star rating, album addition, tagging, favoriting, NSFW toggling, clipboard copying, and batch file moves/copies/deletion.

## Database & Storage Maintenance

- **Real-Time Storage Metrics (`DatabaseManagerModal.vue`)**: Inspects file size, total indexed images, folders, albums, tags, cached hashes, SQLite page count, and free pages.
- **Optimization & Compaction**: Executes `VACUUM` and `PRAGMA optimize` to defragment B-trees and reclaim freelist pages.
- **Point-in-Time Backup & Restore**: Exports snapshots using SQLite `VACUUM INTO` and supports full database restoration with schema and integrity verification.

## Global Keyboard Shortcuts

- `Space` / `Enter`: Open preview / full metadata inspector for selected image.
- `Esc`: Close open modal / preview, or clear active selection.
- `←` / `→` / `↑` / `↓`: Seamlessly navigate images in the virtual grid.
- `Cmd+A` / `Ctrl+A`: Select all images in current view.
- `/` or `Cmd+F` / `Ctrl+F`: Focus search input.
- `1` – `5`: Quick star rating (and `0` to clear).
- `F`: Toggle favorite status.
- `Delete` / `Backspace`: Move selected file(s) to system Trash.
- `?`: Toggle Keyboard Shortcuts Guide modal (`ShortcutsHelpModal.vue`).

## Frontend

- Vue 3 `<script setup lang="ts">` single-file components, built with Vite.
- All Rust interaction goes through `@tauri-apps/api/core` `invoke()`; there is no
  `tauri-plugin-sql` — storage is strictly owned by `berry-storage`.
- `vue-tsc --noEmit` runs type-checking as part of `pnpm build`.

## Conventions

- **Commits**: conventional commits (`feat:`, `fix:`, `chore:`, `docs:`) per
  the branch policy.
- **Branches**: all work branches off `dev` and target `dev` via PR. See
  [BRANCH_POLICY.md](../BRANCH_POLICY.md).
- **Rust**: `rustfmt` defaults, no custom config; `clippy -D warnings` must be
  clean. Unit tests live next to the code (`#[cfg(test)]`).
- **Errors**: domain/crate errors are typed enums (`thiserror` in
  `berry-storage`); the Tauri layer converts them to `String` for IPC.
