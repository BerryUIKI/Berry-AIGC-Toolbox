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

### M1: Scaffolding & Foundation — 🟡 In progress

**Goal**: A runnable shell application with a solid project skeleton.

**Deliverables**:
- Tauri 2 application initialized (Rust core + Vue 3/TS frontend)
- Crate layout for the Rust core (domain / metadata / storage modules)
- SQLite database initialization with schema versioning
- CI pipeline (build + lint + test) for Windows/macOS/Linux
- Directory structure and coding conventions documented

**Legacy mapping**: project infrastructure (former v2.0 M1.1)

---

### M2: Scanning & Indexing — ⬜ Planned

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

### M3: Browsing & Metadata View — ⬜ Planned

**Goal**: Browse the indexed library and inspect metadata.

**Deliverables**:
- Thumbnail grid view with virtualization
- Preview pane (image + metadata panel, toggle with `I`)
- Sorting: creation date, aesthetics score, rating
- Folder view

**Legacy mapping**: viewer, preview, sorting (v1.x viewer)

---

### M4: Search — ⬜ Planned

**Goal**: Find images by metadata.

**Deliverables**:
- Metadata search engine: model name/hash, prompts, parameters
- Reverse hash search against known model list (partial names supported)
- Visual search builder (parameterized queries)
- Search result actions (mark for deletion, remove from database, auto-tag)

**Legacy mapping**: Search GUI, reverse hash search, Tools menu actions (v0.9+)

---

### M5: Organization — ⬜ Planned

**Goal**: Organize and curate the library.

**Deliverables**:
- Albums (add via context menu or drag-and-drop)
- Custom tags
- Favorites and ratings (1–10)
- NSFW: manual tagging, keyword auto-tagging, blur
- Prompt / negative-prompt lists with usage statistics

**Legacy mapping**: albums, tags, favorites, ratings, NSFW (v1.x organization)

---

### M6: Models & File Operations — ⬜ Planned

**Goal**: Manage models and file-level operations.

**Deliverables**:
- Checkpoint filtering by name and hash
- AUTOMATIC1111 cache.json integration (SHA256)
- Drag-and-drop move/copy between folders
- Database backup and restore
- Remaining metadata formats rollout: InvokeAI, NovelAI, Fooocus/FooocusMRE, ComfyUI, EasyDiffusion, Stable Swarm

**Legacy mapping**: Checkpoints tab, drag-and-drop, Database tab (v0.9+)

---

### M7: Localization & Release — ⬜ Planned

**Goal**: Ship a polished, localized product.

**Deliverables**:
- Localization framework + 7 languages (en, fr, es, de, ja, zh-CN, zh-TW)
- Windows / macOS / Linux installers via Tauri bundler
- Release pipeline, changelog, and release notes

**Legacy mapping**: localization, releases (v1.x/v2.0)

## Cross-Cutting Tasks

| Task | Scope |
|------|-------|
| Metadata format rollout | A1111/SDNext in M2; remaining formats in M4–M6 |
| Testing strategy | Unit tests per crate (M2+), E2E smoke tests (M3+) |
| Performance | Virtualized grid (M3), incremental scan (M2+) |

## Release Notes

- **v3.0.0-alpha (planned)**: first usable release after M3 (browse + view)
- **v3.0.0 (planned)**: after M7 (full feature parity + installers)

---

**Last Updated**: 2026-08-27
