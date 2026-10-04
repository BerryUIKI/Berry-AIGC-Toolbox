# Changelog

All notable changes to the Omera project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.2] - 2026-10-04

### Phase 6: Native Image Transcoding, Managed Import Presets & Safe Gallery Optimization

Version 0.4.2 delivers Phase 6 of the Omera roadmap (#118, #182), introducing pure-Rust image transcoding and compression engines, instant export size estimation, configurable vault import optimization presets, non-destructive gallery batch transformation with safe original disposition, and full internationalization across 7 languages:

#### ⚡ Native Image Transcoding & Compression Engines (#118)
- **Pure-Rust AVIF & Lossless WebP**: Built-in support for AVIF encoding (`ravif`) and fast lossless WebP transcoding (`image` / `webp`) in `crates/omera-scan` with tuneable quality and speed tradeoffs.
- **Format Fallback & Reliability**: Graceful fallback strategies during transcoding to ensure zero image corruption and verifiable file outputs.

#### 📊 Export Estimation & Live Preview (#118)
- **Instant Size Estimation (`estimate_export_file`)**: Added fast single-file simulation command calculating exact estimated byte sizes, space delta, and compression percentage before running full batch exports.
- **Export Modal Comparison Card (`ExportModal.vue`)**: Interactive visual before/after comparison showing source size, projected output size, and space savings percentage.

#### 📥 Managed Vault Import Presets & OS Drop Handling (#118)
- **Import Optimization Modal (`ImportTransformModal.vue`)**: Configurable import workflow for managed vaults with resolution downscaling constraints (e.g. max width/height while preserving aspect ratio), target format conversion (WebP, AVIF, JPEG, PNG, Original), AI metadata stripping or preservation, and filename collision handling.
- **Direct Sidebar Import & Drop Routing**: One-click import button on the sidebar and native OS drag-and-drop routing directly to managed vault folders with automatic preset transformation.

#### 🔄 Gallery Batch Transformation with Safe Disposition (#160, #182)
- **Batch Transform Workflow (`BatchTransformModal.vue`)**: Gallery multi-select action to transcode and optimize existing library images in bulk with real-time progress tracking, cancelability, and aggregated savings summaries.
- **Non-Destructive Original Disposition**: Supports three user-chosen policies:
  - `Keep`: Retains the original file alongside the transcoded file.
  - `Archive`: Moves the original into a hidden `.omera_archive/` subfolder preserving relative file structure.
  - `Trash`: Safely relocates the original to the OS system Recycle Bin / Trash without permanent deletion.
- **Linked Folder Read-Only Protection**: External linked library folders are strictly safeguarded against destructive original disposition (`trash` / `archive` disabled, forcing non-destructive output).
- **Metadata & DB Seamless Sync**: Automatically preserves AI generation parameters and updates SQLite database records and thumbnail caches without losing favorites, ratings, or tags.

#### 🌐 Complete 7-Language Internationalization
- **Multi-Language Coverage**: Added localized strings across English (`en`), Simplified Chinese (`zh-CN`), Traditional Chinese (`zh-TW`), Japanese (`ja`), German (`de`), French (`fr`), and Spanish (`es`) for all import, export, and batch transformation dialogs.

---

## [0.4.1] - 2026-10-04

### Theme Contrast & Design Tokens Unification, Aggregated Facet Counts & High-Load Performance Caching

Version 0.4.1 is a maintenance and performance patch release fixing theme contrast issues across dark and light modes, unifying semantic design tokens, aggregating filter/tag statistics into single-pass database queries, and caching distinct metadata models and samplers to eliminate CPU overhead under heavy library load:

#### 🎨 Theme Design Tokens & Contrast Unification (#177, #178)
- **Eliminate Hardcoded Colors**: Replaced all hardcoded dark and light hex values across `SortBar.vue`, `TitleBar.vue`, `SettingsModal.vue`, `TagModal.vue`, `AlbumModal.vue`, and form controls with semantic CSS custom properties (`--color-text-primary`, `--color-text-secondary`, `--color-bg-surface`, `--color-bg-hover`, `--color-bg-card`, `--color-border-subtle`).
- **TitleBar Version Visibility**: Resolved muted version subtitle contrast in Light mode, ensuring crisp readability across all operating systems.
- **Select & Dropdown Contrast**: Fixed native `<option>` background and text contrast across custom select inputs in light mode to prevent unreadable white-on-white text.
- **Automated Theme Regression Test**: Added comprehensive theme contrast tests in `tests/theme-contrast.test.mjs` verifying design token adherence across all components.

#### ⚡ High-Load Facet Aggregation & In-Memory Caching (#180)
- **Single-Query Library Summary Counts**: Replaced separate queries with a unified SQLite query calculating total files, favorites, and NSFW flags simultaneously (`SELECT COUNT(*), COALESCE(SUM(is_favorite), 0), COALESCE(SUM(is_nsfw), 0) FROM files`), cutting startup and navigation IPC latency.
- **Aggregated Tag Counts**: Replaced potential N+1 tag frequency lookups with an indexed `GROUP BY tag_id` aggregation (`get_tag_counts`), directly displaying counts on Sidebar and TagModal chips.
- **Distinct Models & Samplers Facet Cache**: Added thread-safe `RwLock` in-memory caching for `list_distinct_models` and `list_distinct_samplers`, eliminating repetitive full-table JSON parsing across 50,000+ files on repeated drawer opens. Cache is automatically invalidated upon folder scan completion.
- **Sidebar & Tag Badges**: Added live count badges for Favorites and Sensitive navigation items, as well as file count indicators on tag chips.

---

## [0.4.0] - 2026-10-03

### Omera Identity Transition, Single-Instance Lifecycle, Hardened Security & Verified Transformations

Version 0.4.0 is a milestone pre-release completing the official project identity transition to **Omera** (`com.berryuiki.omera`), enforcing single-instance desktop lifecycle, restricting WebView CSP and local protocol boundaries, introducing verified image export and managed vault transformation engines, auto-detecting adult content with user override preservation, and embedding a comprehensive offline feature documentation drawer:

#### 🌟 Omera Canonical Identity & Migration Bridge
- **Canonical App Identity**: Fully transitioned binary naming, application metadata, OS bundles, and packaging to **Omera** (`com.berryuiki.omera`, repository `BerryUIKI/Omera`, primary SQLite database `omera.db`, local preferences prefix `omera_`).
- **Zero-Data-Loss Migration Bridge**: Integrated automatic discovery for legacy Berry AI Studio installations (`berry.db`, `berry_*` local storage keys, and legacy snapshots), migrating them seamlessly into Omera with verifiable receipts (`berry-to-omera-v1`).
- **Core Crate Restructuring**: Reorganized core architectural components under `crates/omera-*` (`omera-domain`, `omera-metadata`, `omera-scan`, `omera-storage`, `omera-tagger`, `omera-clip`).

#### 🪟 Desktop Lifecycle & Security Hardening
- **Single-Instance Enforcement (#124)**: Integrated `tauri-plugin-single-instance = "2"`, preventing duplicate database locks and taskbar clutter by unminimizing and focusing the active application window upon secondary launch.
- **Defense-in-Depth WebView CSP (#107)**: Enforced strict Content Security Policy denying untrusted remote scripts, inline object embeds, and arbitrary frame nesting, while permitting necessary local asset rendering and verified API endpoints.
- **Scoped Local Asset Protocol (#107)**: Removed wildcard scopes (`scope: []`) and dynamically authorized local asset protocol access strictly to user-registered library folders and the managed WebP thumbnail cache.

#### 📤 Verified Export & Transcoding Precision (#158)
- **Format & Extension Agreement**: Synchronized container encoding for `ExportFormat::Original` with resize or privacy options across JPEG, WebP, and PNG, preventing mislabeled file signatures.
- **Strict Privacy Sidecars**: Ensured `.txt` prompt and `.json` metadata sidecars strictly follow effective sanitization policies (`StripAll` completely omits sidecars; `StripAllAiMetadata` scrubs all AI workflow chunks).
- **Collision Suffixing**: Safe collision detection appending numeric increments (`_1`, `_2`) for both directory and ZIP package exports, preventing accidental file overwrites.
- **Accurate Outcome Reporting**: Counted exported items only after verified completion of image and sidecar disk writes, reporting granular per-file errors.

#### 🔄 Staged Image Transformation & Managed Vaults (#159, #160)
- **Managed Import Transformation (T2, #159)**: Optional image compression and conversion during import into managed vaults, with staged decode verification before database indexing; external source images remain unmodified by default.
- **Library Batch Compression & Safe Disposition (T3, #160)**: Gallery batch action to convert and compress existing managed media assets. Supports explicit user-directed original disposition (`Keep`, `Archive` to `.omera_archive/`, or move to OS `Trash` without permanent deletion fallback). External linked folders are strictly read-only.

#### 🔞 Rule-Based NSFW Auto-Detection (#128)
- **Rule-Based Keyword Analysis**: Integrated automated prompt and WD14 tag analysis in `omera-metadata` to flag sensitive content on initial ingest.
- **User Override Absolute Priority**: SQLite upsert policies ensure explicit user curation (`is_nsfw` manual toggle) is permanently preserved across library rescans and folder walks.

#### 📖 Global In-App Help Drawer & Feature Documentation (#137)
- **Offline Help & Feature Guide**: Lightweight, zero-network documentation reader (`HelpGuideDrawer.vue`) featuring 11 feature chapters, quick keyword search, and keyboard shortcut matrix.
- **Smart Context Deep-Linking**: Auto-detects active workspace context (burst stacks, video media, settings, models, CLIP, search) to highlight relevant documentation.
- **Global Keybinding**: Summonable from any screen via `F1` or `Ctrl+Shift+H` / `Cmd+Shift+H`, and accessible via the top MenuBar across all 7 supported UI locales.

#### ⚡ Asynchronous Command Concurrency & Schema Evolution (#102, #135)
- **Asynchronous Command Scheduling**: Offloaded blocking filesystem operations, network requests, and ONNX inference to `tauri::async_runtime::spawn_blocking`, keeping database lock durations minimal.
- **Large-Library Storage Benchmarks**: Verified keyset cursor indexing (`idx_files_cursor_mtime`, `idx_files_cursor_size`, `idx_files_cursor_rating`) and documented query plan metrics across 1k, 10k, and 50k item libraries in `docs/benchmarks/LARGE_LIBRARY_BENCHMARK.md`.

---

## [0.3.0] - 2026-09-21

### Cloud Sync & Export Utilities, Keyset Deep Pagination & Multi-Database Team Studio (Milestones 12, 13, 14)

Version 0.3.0 is a major milestone release delivering batch format conversion and privacy stripping, standalone HTML showcase export, S3 and WebDAV snapshot backup and incremental mirroring, keyset cursor deep pagination for 500,000+ assets, directory fingerprint benchmarking, and enterprise multi-database collaboration support:

#### ☁️ Cloud Sync & Export Utilities (Milestone 12)
- **Batch Transcoding & Packaging (12.1)**: Multi-threaded format conversion to WebP, JPEG, and PNG using Rayon; pure-Rust ZIP archive packaging and directory export.
- **4-Tier Privacy Sanitization (12.1)**: Configurable metadata stripping (`KeepAll`, `StripPromptOnly`, `StripAllAiMetadata`, `StripAll` pixel-only sanitization) removing embedded prompts, generation parameters, and sensitive EXIF/ICC chunks.
- **Filename Pattern Templating & Sidecars (12.1)**: Flexible export naming with tokens (`{name}`, `{date}`, `{model}`, `{id}`, `{rating}`) and optional `.txt` prompt or `.json` metadata sidecars.
- **Standalone Interactive HTML Showcase (12.2)**: Zero-dependency, single `index.html` export containing responsive dark-theme gallery, fullscreen pan/zoom lightbox with keyboard navigation (`Esc`, `←`, `→`), prompt inspector with one-click copying, and instant keyword filter.
- **S3 & WebDAV Snapshot Backup & Restore (12.3)**: Automated and manual snapshot creation targeting AWS S3 (SigV4), Cloudflare R2, MinIO, Backblaze B2, and WebDAV servers (Nextcloud/Synology). Includes hot SQLite `VACUUM INTO`, schema verification, pre-restore rollback backup, and live database connection hot-swapping.
- **Incremental Remote Asset Mirroring & Delta Sync (12.4)**: ETag and streaming SHA-256 change detection for bidirectional asset sync, with a token-bucket `RateLimiter` aggregate bandwidth throttle, atomic cancellation, dry-run simulation, and live animated UI progress.

#### ⚡ Keyset Cursor Deep Pagination & Scalability (Milestone 13)
- **Keyset Cursor Deep Pagination**: Virtual gallery scrolling directly integrated with keyset cursors (`search_files_cursor_page` and `search_files_by_query_cursor_page`), eliminating SQLite `OFFSET N` scans and decoupling window counting for \(O(1)\) row traversal (< 1 ms at 40k+ assets, an 84x speedup over offset paging).
- **Directory Fingerprint Benchmarks**: Automated benchmark suite (`crates/berry-scan/benches/directory_fingerprint.rs`) evaluating 4 directory traversal strategies, showing parent directory mtime gating delivers a 2.5x speedup locally and 5.6x speedup over network shares (SMB/NFS/WebDAV).

#### 👥 Multi-Database Support & Team Studio (Milestone 14)
- **Storage Engine Abstraction**: Abstract `StorageEngine` trait supporting SQLite, MySQL 8.0+, and PostgreSQL 14+ backends.
- **Storage Root Mapping**: Cross-platform path normalization (`storage_roots` table and client mount configurations) for multi-user team collaboration across Windows, macOS, and Linux.
- **Optimistic Concurrency Control**: Row-level version tracking (`version` column) detecting concurrent modifications with last-write-wins and set-union conflict resolution.
- **Real-Time Collaboration Sync Engine**: Zero-DevOps change log journal polling engine (`CollaborationSyncEngine`) with automatic cadence adaptation and cross-client event dispatching.
- **Team & Database Settings**: Dedicated configuration tab with live database connection testing, latency ping diagnostics, and storage root mount mapping.

---

## [0.2.1] - 2026-09-19

### Large-Library Performance & Gallery Virtualization (Milestone 13)

Version 0.2.1 delivers comprehensive performance optimizations across gallery rendering, thumbnail caching, database queries, filesystem indexing, and application bundles to ensure fluid navigation with tens of thousands of local images:

#### ⚡ Virtual Gallery & Layout Optimization
- **Responsive Fixed-Width Columns**: Maintained stable card widths during viewport resizing, dynamically adjusting column counts instead of stretching images.
- **Frame-Coalesced Scrolling**: Coalesced scroll events through `requestAnimationFrame`, limiting reactive updates to at most one per rendered frame.
- **Per-Column Binary Search**: Replaced full dataset filtering in Waterfall view with per-column binary search for visible items.
- **Filter-Aware Stack Summaries**: Kept stack counts, heroes, and expanded members aligned with active text, structured, and semantic search filters.

#### 📦 Incremental Data Delivery & Deferred Payloads
- **Bounded 400-Item Paging**: Replaced full table materialization with bounded 400-item SQLite queries returning exact filtered window counts.
- **Deferred Raw Metadata**: Omitted heavy prompt strings and ComfyUI workflow JSON from gallery page IPC; loaded detailed records on-demand with a 64-entry LRU cache.
- **Stale Response Invalidation**: Discarded in-flight page queries when navigation filters, folders, or sort orders change.

#### 🖼️ Smart Multi-Tier Thumbnail Pipeline
- **Persistent Thumbnail Manifest (Schema v11)**: SQLite-backed manifest tracking image IDs, modification times, size tiers, and codecs with rate-limited access tracking.
- **Configurable 2 GB Cache Budget**: Automated LRU eviction maintaining disk usage within the configured budget during generation and background migration.
- **Rendered-Size Tier Selection**: Selected the smallest sufficient thumbnail tier covering rendered card dimensions across Grid, Waterfall, and compact Table rows.
- **Existing Tier Reuse**: Reused existing larger cached tiers for the same revision to eliminate redundant decoding.
- **Cancelable Decode Priority**: Prioritized visible thumbnails before speculative look-ahead, using monotonic viewport generations to drop stale decoding jobs.

#### 📂 Streaming Ingestion & Filesystem Watcher
- **Native Directory Watcher**: Registered long-lived platform watchers with a durable SQLite journal, 750ms quiet-period coalescing, and targeted path reconciliation.
- **Streaming Folder Scans**: Streamed media entries into bounded database batches without keeping full directory trees in memory.
- **Throttled Scan Progress**: Coalesced scan progress IPC events to at most one per 64 files or 100 ms.
- **Startup Scan Cooldown**: Added a six-hour default cooldown preventing repetitive full-tree walks on app launch.

#### 🚀 Bundle & Resource Reduction
- **Async Modal Bundle Splitting**: Dynamically imported 18 infrequent modals and drawers, reducing the entry JavaScript bundle by ~103 KB and CSS by ~87 KB uncompressed (~25 KB gzipped).

---

## [0.2.0] - 2026-09-16

### AI Semantic Search, LoRA Catalog, Multi-Mode Pipelines & Advanced Stacking

Version 0.2.0 is a major milestone release delivering AI-assisted tagging, local CLIP semantic search, a LoRA trigger word library, multi-mode ingestion pipelines, burst image stacking, stack merge safety, and a virtualized waterfall gallery:

#### 🧠 Local CLIP Semantic Search & AI Tagging (Milestone 8)
- **Local CLIP Vision & Text Embeddings**: Integrated ONNX-powered CLIP/SigLIP embedding pipeline with tokenization and batch indexing dashboard (`ClipManagerModal.vue`).
- **Natural Language Semantic Search**: Direct text-to-image semantic search in the main search bar with similarity scoring.
- **Visual Similarity Search**: Find similar compositions and styles from the context menu with threshold tuning and limit controls.
- **WD14 / Danbooru Anime Tagger**: Automated anime and realistic tag extraction with confidence score thresholds and batch tagging.
- **Persistent Embedding Storage**: SQLite schema v7 `file_embeddings` table with fast cosine similarity ranking.

#### 🔮 LoRA Trigger Library & Parameter Extraction (Milestone 9)
- **LoRA Detection**: Extracted `<lora:name:weight>` from prompts and parsed ComfyUI `LoraLoader` / `LoraLoaderModelOnly` graph nodes.
- **LoRA Trigger Manager**: Manage Civitai trigger words, copy triggers on click, and inspect detected LoRAs in the Property Inspector.
- **Batch Civitai Metadata Sync**: Scan LoRA directories, match model hashes with Civitai cache, and fetch trigger words.

#### 🗃️ Multi-Mode Ingestion Pipelines & Burst Stacking (Milestone 10)
- **Multi-Mode Folder Architecture**:
  - *External Link Mode*: Reference in-place with zero-copy, read-only watch.
  - *Managed Vault Mode*: Dedicated app storage with automated copy/move file management.
  - *AIGC Pipeline Mode*: Automated harvest from WebUI/ComfyUI output folders with debounced ingestion and delayed trash cleanup.
- **Automated Burst Stacking**: Automatically groups generations produced with similar prompts within a short time window.
- **Poker Deck Stack Cards**: Stack cards with count badges, inline click expand/collapse, and Hero Cover designation (`Alt+S`).
- **Side-by-Side Compare Mode**: Compare stack members side-by-side with synchronized zoom and panning (`C`).
- **Flat Stack Merge Safety**: Safe transactional merging of stacks with pre-merge confirmation dialog and user warning suppression preferences.
- **Authoritative Hero Selection**: Eliminates duplicate zero-order stack members, ensuring consistent hero representation across loading, collapse, and card interactions.

#### ▦ Virtualized Waterfall Gallery & Selection Polish
- **Virtualized Waterfall Layout**: New masonry waterfall view alongside the standard grid view, supporting varying image aspect ratios with high-performance virtual rendering.
- **Robust Range Selection**: Predictable Shift-click range selection starting on the very first click, respecting anchor items.
- **Gallery Stability**: Maintains gallery mount state during stack operations to prevent layout jumps.
- **Settings Redesign**: Reorganized settings modal with dedicated Display, Stacking, and Parsers workflow sections.

---

## [0.1.3] - 2026-09-06

### macOS Universal Build & Retina Icon Polish
- Built universal macOS binaries supporting both Apple Silicon (ARM64) and Intel (x86_64).
- Added high-resolution 1024x1024 retina icon assets to `.icns` bundle for Mac App Store compatibility.
- Hardened CI release packaging, Gatekeeper notarization, and bundle discovery scripts.

---

## [0.1.2] - 2026-09-01

### Cross-Platform Packaging & Updater Fixes
- Added automated GitHub Releases updater with SemVer comparison and multi-state feedback.
- Standardized cross-platform release package naming convention across Windows (NSIS, portable zip), macOS (DMG), and Linux (AppImage, deb, rpm).
- Improved SQLite database compaction and backup export workflows.

---

## [0.1.1] - 2026-08-31

### Eagle-Style Studio Workspace & Frameless Window Upgrade

Version 0.1.1 delivers a major UI/UX transformation with a professional Eagle-style three-pane studio layout, custom frameless window, integrated desktop menu bar, and GitHub Releases auto-update features:

#### 🎨 Eagle-Style Three-Pane Studio Layout
- **Frameless Window**: Custom window title bar with drag region, native-like minimize, maximize/restore, and close controls.
- **Top Desktop MenuBar**: Integrated `File`, `Edit`, `View`, `Tools`, and `Help` dropdown menus with single-language i18n support.
- **Left Navigation Sidebar**: Redesigned library groups (All Images, Favorites, Sensitive 18+), folder tree with real-time scan state, smart albums, and color-coded tags.
- **Center Gallery Canvas**: Virtual scrolling grid with dynamic column resizing, smooth thumbnail zoom slider (130px–360px), and Grid (⊞) / Table (☰) view switcher.
- **Right Property Inspector**: Dedicated inspector pane with large preview cards, star rating (0–5), favorite toggle, tokenized prompt chips with one-click copy, model specs, and raw workflow JSON tree viewer.
- **Full-Screen Lightbox (Quick Look)**: Immersive modal viewer with mouse-wheel zoom, pan, and keyboard arrow navigation.

#### 🚀 GitHub Releases Auto-Updater
- **Check for Updates**: Direct update check from `Help > Check for Updates...` via GitHub Releases API.
- **Multiple Update States**: Real-time status indication for `Checking`, `Up to date`, `Update available` (with release notes & direct installer downloads), `Development / Ahead of Release`, and `Error`.

#### 🌐 Localization & Pre-Release Polish
- **Auto System Language Detection**: Added `Auto (System)` option in i18n to automatically track OS language.
- **Multilingual Documentation**: English, Simplified Chinese, Traditional Chinese, and Japanese READMEs with quick language switchers.
- **Standardized Release Asset Naming**: Standardized multi-platform release artifacts convention: `<AppName>_<OS>_<Architecture>.<extension>`.
- **UI Platform Badges**: Normalized generator badge labels to `WebUI`, `ComfyUI`, `NovelAI`, `Fooocus`, `InvokeAI`, and `SD`.

---

## [0.1.0] - 2026-08-31

### Complete Clean-Slate Rewrite (Tauri 2 + Rust + Vue 3)

Version 0.1.0 marks the initial release of the complete clean-slate rewrite of Berry AI Studio from the legacy C#/.NET architecture to a modern, high-performance, cross-platform stack powered by Tauri 2, Rust core crates, and Vue 3 + TypeScript.

---

### Features by Milestone

#### Milestone 1: Scaffolding & Foundation
- Modular Rust multi-crate Cargo workspace layout (`berry-domain`, `berry-metadata`, `berry-scan`, `berry-storage`, and `berry-ai-studio`).
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
