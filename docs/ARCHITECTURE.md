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

## Schema versioning

`berry-storage` tracks schema versions with SQLite's `PRAGMA user_version`.
`crates/berry-storage/src/migrations.rs` holds an ordered `MIGRATIONS: &[&str]`;
`Database::migrate()` applies each pending migration inside a transaction and
bumps `user_version` by one.

**Rules:**

- Never edit, reorder, or delete an applied migration — deployed databases
  depend on the sequence. Add a new entry to the end of `MIGRATIONS` instead.
- Every schema change ships with a migration, never ad-hoc `CREATE`/`ALTER`
  statements.

## Browsing, Asset Protocol & Virtualization
 
- **Asset Protocol**: Configured via Tauri 2 `app.security.assetProtocol` (`protocol-asset` feature enabled). Local images are streamed into `<img>` tags via `@tauri-apps/api/core` `convertFileSrc`, with Windows verbatim prefixes (`\\?\`) sanitized before resolution.
- **Virtualized Grid (`VirtualGrid.vue`)**: Custom responsive virtualization engine. `ResizeObserver` dynamically adapts column count to container width (`minWidth: 180px`). Total height is driven by a phantom element; only rows in the visible viewport (plus 2-row overscan buffer) are rendered in the DOM to ensure smooth 60fps scrolling over thousands of items without memory leaks.
- **Navigation & Sorting**: `query_files` handles optional folder filtering and multi-column sorting (date, path, size, rating with NULLs last, aesthetic score with NULLs last).
- **Preview & Inspector (`PreviewPane.vue`)**: Modal dialog with full-resolution image viewer, keyboard navigation (`←`/`→`/`Esc`), interactive rating widget (1–10 stars), prompt copy buttons, and collapsible metadata inspector toggled by the `I` shortcut.

## Search Engine, Query Syntax & Batch Actions

- **Search Criteria & Engine**: Defined by `SearchCriteria` in `berry-domain` and executed by `search_files` in `berry-storage`. Utilizes parameterized SQL with SQLite `json_extract()` for indexing parameters embedded in JSON text (`prompt`, `negative_prompt`, `model_name`, `model_hash`, `sampler`, `steps`, `cfg_scale`), alongside table columns (`rating`, `aesthetic_score`, `folder_id`).
- **Query Parser**: `parse_query` in `berry-domain::search_parser` parses free-form strings into structured criteria:
  - Key-value tokens: `prompt:...`, `neg:...`, `model:...`, `hash:...`, `sampler:...`
  - Quoted string support: `model:"dreamshaper xl"`, `prompt:"neon cat"`
  - Numeric ranges: `steps:20..40`, `cfg:5.0..8.5`
  - Comparison operators: `steps:>=25`, `rating:>=8`, `cfg:<10`
  - Bare words are automatically aggregated into broad `text` substring matching across prompt, negative prompt, model name, and path.
- **Visual Filters (`FilterDrawer.vue`)**: Slide-out drawer with dynamic checkpoint and sampler dropdowns queried from `list_distinct_models` and `list_distinct_samplers`, bidirectional synchronization with the search input via `criteriaToQuery`, and active filter count badges.
- **Batch Selection & Toolbar (`BatchActionBar.vue`)**: Multi-selection via checkboxes, `Cmd+Click`, `Shift+Click` range selection, and `Cmd+A` keyboard shortcut. Provides high-performance batch operations: single-transaction multi-file rating updates (`set_files_rating`), newline-separated path copying, and prompt extraction.

## Frontend

- Vue 3 `<script setup lang="ts">` single-file components, built with Vite.
- All Rust interaction goes through `@tauri-apps/api` `invoke()`; there is no
  `tauri-plugin-sql` — storage is owned by `berry-storage`.
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
