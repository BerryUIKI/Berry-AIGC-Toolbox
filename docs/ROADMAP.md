# Development Roadmap

Berry-AIGC-Toolbox clean-slate rewrite (Tauri 2 + Rust + Vue 3).

Milestones are **feature-first**: each milestone delivers a runnable slice of
the application that builds on the previous one. Status is tracked per
milestone; see the legend below.

## Status Legend

- ⬜ Planned — not started
- 🟡 In progress — actively being worked on
- ✅ Complete — done and merged to `dev`

## Milestones

### M1: Scaffolding & Foundation — ✅ Complete

**Goal**: A runnable shell application with a solid project skeleton.

**Deliverables**:
- Tauri 2 application initialized (Rust core + Vue 3/TS frontend)
- Crate layout for the Rust core (domain / metadata / storage modules)
- SQLite database initialization with schema versioning
- CI pipeline (build + lint + test) for Windows/macOS/Linux
- Directory structure and coding conventions documented

**Legacy mapping**: project infrastructure (former v2.0 M1.1)

---

### M2: Scanning & Indexing — ✅ Complete

**Goal**: Index images and extract metadata into the database.

**Deliverables**:
- Folder selection and management
- Recursive scan of images/videos (JPG/PNG/WebP/MP4)
- Metadata extraction: PNGInfo, EXIF, .TXT sidecar files
  - First batch: AUTOMATIC1111 / SDNext
- SQLite persistence of images and metadata
- Metadata rebuild command (re-scan existing files)

**Legacy mapping**: image scanning, PNGInfo indexing, rebuild metadata (v1.x core)

---

### M3: Browsing & Metadata View — ✅ Complete

**Goal**: Browse the indexed library and inspect metadata.

**Deliverables**:
- Thumbnail grid view with virtualization (`VirtualGrid.vue`)
- Preview pane with metadata inspector and prompt copying (toggle with `I`)
- Multi-criteria sorting: creation date, file name, size, rating, aesthetics score
- Folder view and "All Images" library navigation with live counts
- Interactive rating updates preserved across scans

**Legacy mapping**: viewer, preview, sorting (v1.x viewer)

---

### M4: Search — ✅ Complete

**Goal**: Find images by metadata with text queries, visual filters, and batch actions.

**Deliverables**:
- Search domain models (`SearchCriteria`) and dynamic parameterized SQLite search engine (`berry-storage`)
- Query string parser supporting key-value tokens (`prompt:`, `model:`, `neg:`, `sampler:`), quoted text, comparison operators (`>=`, `<=`), and numeric ranges (`steps:20..40`)
- Global search bar with 250ms debouncing, hotkey focus (`/`, `Cmd+F`), removable filter chips, and instant matching counts
- Visual filter drawer (`FilterDrawer.vue`) with dynamic checkpoint model & sampler dropdowns, star rating ranges, and generation parameter ranges
- Multi-selection grid/table actions with floating toolbar: batch star rating (`set_files_rating`), clipboard copying for paths and prompts, and `Cmd+A` keyboard selection

**Legacy mapping**: Search GUI, reverse hash search, Tools menu actions (v0.9+)

---

### M5: Organization — ✅ Complete

**Goal**: Organize and curate the library.

**Deliverables**:
- Albums (custom collections, batch assignment, sidebar navigation)
- Custom tags (color-coded chips, batch tagging, search filtering)
- Favorites and ratings (1–10 stars) with batch actions
- NSFW: manual tagging, blur overlay with click-to-reveal
- Prompt / negative-prompt insights modal (`PromptStatsModal.vue`) with keyword frequencies and quick-search links

**Legacy mapping**: albums, tags, favorites, ratings, NSFW (v1.x organization)

---

### M6: Performance, Cache & Polish — ✅ Complete

**Goal**: Manage models, multi-format metadata parsers, file-level operations, database maintenance, and keyboard shortcuts.

**Deliverables**:
- Multi-format metadata parsers: ComfyUI (node graphs), NovelAI (Comment JSON), InvokeAI (sd-metadata/invokeai_metadata), Fooocus (parameters text), EasyDiffusion, and Stable Swarm
- Checkpoint model catalog & short hash / SHA256 resolution (`ModelManagerModal.vue`, A1111 `cache.json` import)
- File-level operations: move files, copy files, safe trash deletion (cross-platform OS trash integration), and reveal in system file manager (Finder / Explorer / Files)
- Drag-and-drop from grid cards directly onto sidebar Folders, Albums, and Tags
- Database maintenance: `VACUUM` & optimize, point-in-time backup snapshot export, integrity verification & restore (`DatabaseManagerModal.vue`)
- Global keyboard shortcuts (`Space` preview, `Esc`, Arrow navigation, `Cmd/Ctrl+A`, `1-5` rating, `F` favorite, `Delete` trash, `?` guide modal)

**Legacy mapping**: Checkpoints tab, drag-and-drop, Database tab (v0.9+)

---

### M7: Localization & Release — ✅ Complete

**Goal**: Ship a polished, localized product.

**Deliverables**:
- Localization framework + 7 languages (en, fr, es, de, ja, zh-CN, zh-TW) with `LanguageSelector.vue`
- Windows / macOS / Linux packaging configurations via Tauri bundler
- Release pipeline, version bump (3.0.0), changelog, and release notes

**Legacy mapping**: localization, releases (v1.x/v2.0)

## Cross-Cutting Tasks

| Task | Scope |
|------|-------|
| Metadata format rollout | Multi-format parsers across A1111, ComfyUI, NovelAI, InvokeAI, Fooocus, EasyDiffusion |
| Testing strategy | Unit & integration tests per crate (99 total tests passing, 0 failures) |
| Performance | Virtualized grid, incremental scanning, optimized SQLite transactions & indexing |

## Release Notes

- **v3.0.0**: Complete clean-slate rewrite release with full feature parity, modern UI, multi-format metadata support, 7 languages, and cross-platform desktop installers.

---

**Last Updated**: 2026-08-31
