# Berry-AIGC-Toolbox

[![License](https://img.shields.io/badge/license-AGPL--3.0-blue.svg)](LICENSE)
[![Tauri](https://img.shields.io/badge/Tauri-2-24c8db)](https://tauri.app)
[![Rust](https://img.shields.io/badge/Rust-1.7+-orange)](https://www.rust-lang.org)
[![Vue](https://img.shields.io/badge/Vue-3-42b883)](https://vuejs.org)

Berry-AIGC-Toolbox is an open-source **metadata indexer and viewer for
AI-generated images**. It helps you organize, search, and manage your growing
collection of AI-generated content — prompts, models, and metadata — across
multiple AI platforms and file formats.

> This project is a **clean-slate rewrite** (Tauri 2 + Rust + Vue 3) of the
> original Berry-AIGC-Toolbox. The legacy C#/.NET codebase is archived in the
> `old/main` branch for reference only.

## Features

### Scanning & Indexing
- Scan folders for images and videos, store and index prompts and other metadata (PNGInfo, EXIF, .TXT sidecars)
- Rebuild metadata on demand (re-scan and re-extract from existing files)
- Folder-based organization with drag-and-drop file management

### Supported Metadata Formats
- AUTOMATIC1111 / SDNext
- InvokeAI (Dream / sd-metadata / invokeai_metadata)
- NovelAI
- Stable Diffusion
- Fooocus / FooocusMRE
- ComfyUI
- EasyDiffusion
- Stable Swarm

### Supported File Formats
- JPG / JPEG (+ EXIF)
- PNG
- WebP
- MP4 (video)
- .TXT sidecar metadata files

### Organization
- Albums (custom collections, batch assignment, sidebar navigation)
- Custom tags (color-coded chips, batch tagging, search filtering)
- Favorites and ratings (1–10 stars)
- NSFW tagging: manual, blur overlay with click-to-reveal

### Search & Prompts
- Advanced metadata search (model file name/hash, prompts, parameters, and more)
- Visual search builder for fine-grained queries
- Prompt and negative-prompt insights modal (`PromptStatsModal.vue`) with keyword frequencies and average ratings
- Reverse hash search against a known model list (partial name matching supported)

### Sorting & Navigation
- By creation / modification date, file name, file size
- By rating (with unrated files positioned last)
- By aesthetics score (with unrated files positioned last)
- Grid view (`VirtualGrid.vue` with dynamic responsiveness and infinite scrolling) and Table view (`FileList.vue`)
- Global keyboard navigation (`Space`/`Enter` preview, `Esc` deselect, Arrow key navigation, `Cmd+A` select all, `1-5` rating, `F` favorite, `Delete` trash, `?` shortcuts guide)

### Model Management
- Checkpoint models catalog and image counts browser (`ModelManagerModal.vue`)
- Filter checkpoints by name and hash
- AUTOMATIC1111 `cache.json` and custom dictionary integration (SHA256 and short hash lookup)

### File Operations
- Move and copy files (and associated `.txt`/`.json` sidecars) between indexed folders
- Drag-and-drop from grid view directly onto sidebar Folders, Albums, and Tags
- Safe deletion to system Trash / Recycle Bin via native OS integration
- Instant file reveal in system file manager (Finder / Explorer / Files)

### Localization
- English, French, Spanish, German, Japanese, Simplified Chinese, Traditional Chinese

### Database & Maintenance
- Real-time database metrics dashboard (file size, SQLite page allocations, record counts)
- Point-in-time database backup snapshot export via SQLite `VACUUM INTO`
- Database restore with SQLite integrity validation
- One-click `VACUUM` compaction and query optimization

## Tech Stack

| Component        | Technology                         |
|------------------|------------------------------------|
| Desktop Shell    | [Tauri 2](https://tauri.app)       |
| Core Logic       | [Rust](https://www.rust-lang.org)  |
| Frontend         | [Vue 3](https://vuejs.org) + TypeScript |
| Database         | SQLite                             |
| Packaging        | Tauri bundler (Windows / macOS / Linux) |

## Roadmap

The rewrite is delivered in feature-first milestones — each milestone produces
a runnable slice of the application.

| # | Milestone                    | Outcome                                   | Status  |
|---|------------------------------|-------------------------------------------|---------|
| M1 | Scaffolding & Foundation     | Runnable shell app, core crate, CI, SQLite | ✅ |
| M2 | Scanning & Indexing          | Folder scan, PNGInfo/EXIF/TXT extraction, storage | ✅ |
| M3 | Browsing & Metadata View     | Thumbnail grid, preview pane, metadata panel, sorting | ✅ |
| M4 | Search                       | Metadata search engine, query parser, visual filters, batch actions | ✅ |
| M5 | Organization                 | Albums, tags, favorites, ratings, NSFW, prompt stats | ✅ |
| M6 | Performance, Cache & Polish  | Checkpoints, file ops, drag-and-drop, DB backup/restore, shortcuts | ✅ |
| M7 | Localization & Release       | 7 languages, Windows/macOS/Linux installers | ✅ |

See [docs/ROADMAP.md](docs/ROADMAP.md) for the detailed milestone breakdown,
deliverables, and status tracking.

## Repository Structure

| Branch      | Purpose                                                             |
|-------------|---------------------------------------------------------------------|
| `main`      | Protected default branch — production-ready code (PR-only)          |
| `dev`       | Development integration branch — feature work lands here first      |
| `old/main`  | Read-only archive of the legacy C#/.NET codebase                    |

All changes flow through Pull Requests. See
[BRANCH_POLICY.md](BRANCH_POLICY.md) for the full branching strategy.

## Development

### Prerequisites
- [Rust toolchain](https://www.rust-lang.org/tools/install) (stable)
- [Node.js](https://nodejs.org) LTS
- [pnpm](https://pnpm.io)
- Tauri 2 prerequisites for your platform (see [Tauri docs](https://tauri.app))

### Commands

```bash
pnpm install          # install frontend dependencies
pnpm tauri dev        # run the desktop app in development (hot reload)
pnpm build            # type-check (vue-tsc) and build the frontend to dist/
pnpm tauri build      # build a release bundle (installer)

cargo test --workspace    # run all Rust tests
cargo clippy --workspace --all-targets -- -D warnings   # lint (must be clean)
cargo fmt --all -- --check                # check formatting
```

Run cargo commands from the repository root — a single workspace covers the
`src-tauri` app crate and the `crates/*` core crates.

### Architecture

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the crate layout, data
flow, and schema-versioning rules.

### Contribution Workflow
1. Create a feature branch from `dev` (`feature/your-feature`)
2. Implement with tests
3. Open a Pull Request targeting `dev`
4. After merge to `dev`, a release PR merges `dev` into `main`

## License

This project is licensed under the **GNU Affero General Public License v3.0**
(AGPL-3.0) — see the [LICENSE](LICENSE) file for details.

---

**Project Status**: ✅ Production Ready (v0.1.0 — All 7 Milestones Completed)
**Current Branch**: `dev`
**Last Updated**: 2026-08-31
