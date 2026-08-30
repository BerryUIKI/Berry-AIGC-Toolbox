# Changelog

All notable changes to the Berry-AIGC-Toolbox project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-08-31

### Complete Clean-Slate Rewrite (Tauri 2 + Rust + Vue 3)

Version 0.1.0 marks the initial release of the complete clean-slate rewrite of Berry-AIGC-Toolbox from the legacy C#/.NET architecture to a modern, high-performance, cross-platform stack powered by Tauri 2, Rust core crates, and Vue 3 + TypeScript.

---

### Features by Milestone

#### Milestone 1: Scaffolding & Foundation
- Modular Rust multi-crate Cargo workspace layout (`berry-domain`, `berry-metadata`, `berry-scan`, `berry-storage`, and `berry-aigc-toolbox`).
- SQLite storage engine with transactional versioning via `PRAGMA user_version` (`migrations.rs`).
- CI/CD build and verification pipelines for Windows, macOS, and Linux.

#### Milestone 2: Scanning & Indexing
- High-performance recursive directory scanner with container detection (`JPG`, `PNG`, `WebP`, `MP4`).
- Incremental indexing skipping unchanged files by `(size, modified_at)` and forced metadata rebuild support.
- Native PNGInfo text chunk extraction and EXIF parsing with `.txt` sidecar fallback.

#### Milestone 3: Browsing & Metadata View
- Responsive virtualized grid view (`VirtualGrid.vue`) rendering thousands of images with 60fps scrolling and memory efficiency.
- Preview & Metadata Inspector dialog (`PreviewPane.vue`) with full-resolution zoom, navigation (`←`/`→`/`Esc`), star ratings (1–10), and one-click prompt copying.
- Multi-criteria sorting by creation date, file name, size, rating, and aesthetics score.

#### Milestone 4: Search Engine & Batch Actions
- Parameterized SQLite search engine (`berry-storage`) with JSON parameter extraction (`prompt`, `negative_prompt`, `model_name`, `model_hash`, `sampler`, `steps`, `cfg_scale`).
- Free-form search query parser (`search_parser.rs`) supporting key-value tokens, quotes, comparison operators, and ranges (`steps:20..40`).
- Visual filter drawer (`FilterDrawer.vue`) with dynamic checkpoint and sampler dropdowns.
- Floating batch actions toolbar (`BatchActionBar.vue`) with multi-selection, batch ratings, and clipboard copying.

#### Milestone 5: Organization & Insights
- Custom Albums (`AlbumModal.vue`) with batch assignment and sidebar collection counts.
- Color-coded taxonomy Tags (`TagModal.vue`) with batch tagging and search filtering (`tag:anime`).
- Favorites and NSFW privacy blur overlays with click-to-reveal.
- Prompt & Metadata Insights (`PromptStatsModal.vue`) analyzing top keyword distributions, average ratings, top models, and samplers.

#### Milestone 6: Performance, Cache & Polish
- Multi-format metadata parsers for **ComfyUI** (node graph JSON), **NovelAI** (Comment JSON), **InvokeAI** (`sd-metadata`), **Fooocus** (parameters text), **EasyDiffusion**, and **Stable Swarm**.
- Checkpoint Model Catalog & Hash Cache (`ModelManagerModal.vue`) supporting AUTOMATIC1111 `cache.json` import and bidirectional hash resolution.
- File-level operations: moving, copying, safe deletion to system Trash / Recycle Bin (`trash` crate), and reveal in system file manager (Finder / Explorer / Files).
- HTML5 Drag-and-Drop from virtual grid directly onto sidebar Folders, Albums, and Tags.
- Database maintenance dashboard (`DatabaseManagerModal.vue`) with real-time metrics, one-click `VACUUM` compaction, and point-in-time backup export/restore (`VACUUM INTO`).
- Global keyboard navigation (`Space`/`Enter` preview, `Esc`, Arrow keys, `Cmd+A`, `1-5` ratings, `F` favorite, `Delete` trash, `?` guide).

#### Milestone 7: Localization & Release
- Reactive i18n localization framework with 7 supported languages:
  - English (`en`)
  - Simplified Chinese (`zh-CN`)
  - Traditional Chinese (`zh-TW`)
  - Japanese (`ja`)
  - German (`de`)
  - French (`fr`)
  - Spanish (`es`)
- Header Language Selector dropdown (`LanguageSelector.vue`) with automatic OS language detection and persistent storage.
- Desktop bundle configuration and installers for macOS (.dmg / .app), Windows (.msi / .exe), and Linux (.deb / .AppImage).
