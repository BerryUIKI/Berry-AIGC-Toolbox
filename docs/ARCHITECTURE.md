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
┌──────────────▼───────────────┐   ┌───────────────────────────────┐
│  berry-domain                │   │  berry-metadata               │
│  shared domain types         │──▶│  format detection & parsing   │
│  (ImageFile, MetadataFormat) │   │  (full parsers land in M2)    │
└──────────────┬───────────────┘   └───────────────────────────────┘
               │
┌──────────────▼───────────────┐
│  berry-storage               │   SQLite persistence: schema
│  Database, migrations        │   versioning via PRAGMA user_version
└──────────────────────────────┘
```

## Crate layout

A single Cargo workspace at the repository root (`Cargo.toml`). Run `cargo test`,
`cargo clippy`, and `cargo fmt` from the root to cover every crate at once.

| Crate | Responsibility | Depends on |
|---|---|---|
| `berry-aigc-toolbox` (`src-tauri/`) | Tauri app shell: window setup, IPC commands, application state. **No business logic** — commands are thin adapters over the core crates. | berry-storage |
| `berry-domain` | Pure domain types shared across crates: `ImageFile`, `Container`, `MetadataFormat`. Depends on nothing in this repo. | serde |
| `berry-metadata` | Detecting and parsing generation metadata (PNGInfo, EXIF, `.txt` sidecars). M1 ships `detect_container` (magic-byte sniffing); per-format parsers arrive in M2. | berry-domain |
| `berry-storage` | SQLite connection + schema versioning. All schema changes go through the ordered `MIGRATIONS` list; there is no ad-hoc DDL. | berry-domain, rusqlite |

## Data flow

1. The frontend calls a `#[tauri::command]` (e.g. `get_app_info`) via
   `invoke()`.
2. The command locks shared state, calls into a core crate, and returns a
   serde-serializable value (or a `Result` mapped to a `String` error).
3. Long-lived resources (the SQLite `Database`) live in Tauri-managed state
   (`AppState` in `src-tauri/src/lib.rs`), opened once during `.setup()` in the
   OS app-data directory.

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
