# Omera product and engineering review

Date: 2026-10-05. Reviewed branch: `dev`, commit `bdac8fb` (`feat(history): implement action history engine with undo/redo for batch operations (#199) (#231)`). Application version: 0.4.3; SQLite schema: 15. Review host: Windows, PowerShell, Node 20.18.0.

Follow-up: issue-template PR #242 is merged into dev at a4ac24f. The original finding groups below describe the initial audit. Their current, narrowed GitHub reports and later findings are indexed in [GITHUB_PUBLICATION_2026-10-05.md](GITHUB_PUBLICATION_2026-10-05.md): 66 individual reports, 21 P1 and 45 P2. [REAL_SAMPLE_TESTS_2026-10-05.md](REAL_SAMPLE_TESTS_2026-10-05.md) records the authorized 1,276-image dataset tests, copied-media operations, source-hash checks, folder initial-scan defect, paginated selection defect, and negative-prompt classification defect. Aggregate issue #265 has been retired; its closure does not mean the findings are fixed.

## Executive assessment

Omera has a useful local-library foundation: multiple gallery modes, broad AIGC metadata parsing, search, stacks, tagging, local SQLite storage, and a thoughtful thumbnail/virtualization design. The main release risk is inconsistent safety across workflows. Ordinary file operations use no-clobber publication and journaling, and local database restore uses a staged lifecycle; newer pipeline, transform, migration, and cloud paths bypass parts of those protections.

From a Product Manager perspective, the next milestone should make the existing promises reliable. Privacy modes, metadata preservation, migration, backup scheduling, and sync correctness need acceptance criteria tied to observable results. Several enabled controls or advertised features are only partially wired. Expanding the feature list before addressing these gaps would increase support and trust problems.

From a CTO perspective, I would gate a broadly distributed release on the P1 findings below. The code builds and existing suites pass, but those suites do not exercise several important error, collision, and recovery paths. This is a release-readiness recommendation, not a claim that every browsing operation is unsafe or that every reported code path has caused real data loss.

## Scope, evidence, and limits

Reviewed Vue UI/state/configuration, Rust command adapters, SQLite persistence/migrations/recovery, metadata/export/transform/scanner services, legacy migration, cloud backup/sync, update trust, CI, and product/engineering documentation. Native interaction was performed on the app launched by `pnpm run tauri dev` from the repository root on `dev`, with executable `D:\dev\Omera\target\debug\omera.exe` and WebView URL `http://localhost:1420/`. An earlier installed-app observation was superseded after the user's correction; the UI findings here were checked against the development app.

Evidence labels:

- **Reproduced:** calls actual local production functions/classes with synthetic temporary fixtures.
- **Observed UI:** native app interaction/accessibility tree on this development build.
- **Code-confirmed:** the relevant caller and implementation were traced; the destructive or remote scenario was not executed against the user's library.
- **Latent:** a public helper or registered backend command has a defect, but the current Vue workflow does not call that path.

The initial review made no production source changes. Reproductions and this report are review artifacts. The findings were subsequently published in [tracking issue #265](https://github.com/BerryUIKI/Omera/issues/265): 32 new reports and 3 reopened issues cover 35 actionable items, with the three R29 topics split for independent delivery. English issue forms and reporting guidance are in [draft PR #242](https://github.com/BerryUIKI/Omera/pull/242), targeting dev. See the [publication record](D:/dev/Omera/docs/reviews/GITHUB_PUBLICATION_2026-10-05.md). Destructive scenarios were confined to synthetic temporary files and databases. Native browsing used the existing 59-item library without changing its assets, ratings, tags, favorites, or NSFW flags. GUI evidence covers Grid, Waterfall, Table, stack expansion, Table selection/Arrow Down navigation, the statistics dialog, menus, and normal/maximized windows. It does not establish 50,000-item WebView performance, full 960px/minimum-width acceptance, reduced-motion behavior, or cross-platform GUI correctness. Real S3/WebDAV transfers, credentials, installers, and signed update downloads were not exercised.

## Verification results

| Check | Result | Meaning |
| --- | --- | --- |
| `pnpm run build` | PASS | TypeScript/Vue typecheck and Vite production build succeed |
| `pnpm run test:stack` | PASS, 154 tests | Existing selected frontend/contract/helper suites pass |
| Four frontend suites omitted from `test:stack` | PASS, 7 tests | Auto-tag mounting/actions, removal confirmation, storage backend checks pass |
| `cargo test --workspace` | PASS, 244 unit tests | App 24, CLIP 2, domain 36, metadata 65, scan 37, storage 77, tagger 3; doctests have no additional tests |
| `cargo clippy --workspace -- -D warnings` | PASS | Requested workspace lint check passes; CI's additional `--all-targets` variant was not separately run |
| `cargo fmt --check` | FAIL | Formatting diffs at `commands.rs` lines 2770, 2819, 2838 |
| `node scripts/generate-ipc-reference.mjs --check` | FAIL | IPC reference is stale |
| Review Rust acceptance probes | FAIL, 10/10 | Deliberate safety/correctness expectations expose the failures listed below |
| Review history acceptance probes | FAIL, 2/2 | Failed undo/redo lose their command |
| `pnpm run tauri dev` | RUNNING successfully | Current development app was inspected |

The Vite build warns about a 580.67 kB main JavaScript chunk (195.63 kB gzip). This is an optimization opportunity, not a measured startup regression. Review probes have an independent dependency lockfile and call the repository's actual local source; they do not use the application's root lockfile. Their direct failure messages agree with the inspected source. The [reproduction package](D:/dev/Omera/docs/reviews/repro-2026-10-05/README.md) contains commands, synthetic fixtures, and the expected/observed matrix.

P1 means high-impact data/privacy/recovery or a materially misleading safety feature to resolve before broad release. P2 means important correctness, usability, completeness, performance, or engineering work. Findings are grouped by user impact; multiple failing probes can support one finding.

## Priority findings

### R01 — P1: Cloud restore bypasses the safe database restore lifecycle

**Reproduced and code-confirmed.** [cloud_backup.rs:903](D:/dev/Omera/src-tauri/src/cloud_backup.rs:903) makes a raw rollback copy of only the live main database, ignores backup errors, and copies the restored file directly over the active database. [commands.rs:4739](D:/dev/Omera/src-tauri/src/commands.rs:4739) calls this while `AppState` and workers still hold database connections; restart occurs only afterward. The database enables a 256 MB mmap at [db.rs:177](D:/dev/Omera/crates/omera-storage/src/db.rs:177).

With a synthetic active WAL connection, this Windows build fails with `Failed to replace live database ... user-mapped section open (os error 1224)`. No real database was restored. On systems allowing the copy, the source path still has active connections/WAL and non-atomic replacement risk; corruption or WAL contamination was not demonstrated on this host. Local restore already uses `recovery::stage_restore`, so cloud restore should use the same lifecycle. Acceptance: validate integrity/schema/FKs, preserve a consistent rollback snapshot, stage restoration, quiesce through restart, apply before opening connections, and test interruption/error recovery.

### R02 — P1: Pipeline harvesting can overwrite a different existing asset

**Code-confirmed.** [commands.rs:4250](D:/dev/Omera/src-tauri/src/commands.rs:4250) considers an existing destination with the same basename and byte length already ingested; equal length does not establish identical content. If lengths differ, [commands.rs:4258](D:/dev/Omera/src-tauri/src/commands.rs:4258) calls `fs::copy` onto the existing file, silently overwriting it. The `.txt` sidecar has the same overwrite behavior. A subsequent upsert can associate the replacement with the original record and its organization.

Reproduction condition: two generators produce distinct `output.png` files into a pipeline whose managed destination already contains that name. Fix through a common no-clobber publication service, explicit collision policy, content verification, and per-item receipts. Acceptance: identical content may deduplicate; different content must survive under separate names without overwriting files or sidecars.

The plain managed-import fallback at [commands.rs:884](D:/dev/Omera/src-tauri/src/commands.rs:884) also assumes that same basename plus equal byte length means identical content. It reuses the existing file/record instead of copying a distinct equal-length asset. This path renames differing-length collisions, so its defect is silent false deduplication rather than the pipeline's overwrite behavior. Include both import paths in the collision acceptance tests.

### R03 — P1: Deferred pipeline cleanup does not revalidate the destination or source identity

**Code-confirmed.** [commands.rs:4331](D:/dev/Omera/src-tauri/src/commands.rs:4331) trashes the queued source pathname once its deadline arrives. It does not use `target_file_id` to check that the managed copy still exists and matches, or verify that the source pathname still represents the file originally ingested. Cleanup is invoked during application startup.

If the destination disappears before the grace period ends, cleanup can remove the remaining original. If a generator replaces the source with a new file at the same path, cleanup can remove that newer file instead. Acceptance: store a source/destination fingerprint, validate both immediately before trashing, retain an actionable failed/deferred state, and require the configured ingestion choice to remain applicable. Never extend this cleanup to external vaults or unrelated media.

### R04 — P1: Legacy configuration migration can fail and still be recorded as successful

**Code-confirmed.** [legacy_migration.rs:254](D:/dev/Omera/src-tauri/src/legacy_migration.rs:254) loads the legacy config and ignores both credential migration and destination save errors. [config_store.rs:99](D:/dev/Omera/src-tauri/src/config_store.rs:99) rejects saving a nonzero legacy revision into an absent destination whose default revision is zero. The coordinator still appends a `config` artifact with status `success` even when no destination config exists.

Additionally, `config_store::load` may rewrite the legacy source to protect plaintext secrets, so this supposedly source-preserving migration reads through a mutating API. Cleanup preview at [legacy_migration.rs:389](D:/dev/Omera/src-tauri/src/legacy_migration.rs:389) marks config and models eligible after checking database health, without validating those destination artifacts. Current Vue code has no cleanup consumer, so cleanup exposure is latent; the migration false-success path itself is active. Acceptance: read legacy data without mutation, translate/reset revision through a migration-specific transaction, propagate failures, verify every artifact, and authorize cleanup only for independently verified destination data and a separate in-app decision.

### R05 — P1: Ambiguous or failed legacy migration is hidden behind a newly created empty library

**Code-confirmed.** [lib.rs:66](D:/dev/Omera/src-tauri/src/lib.rs:66) ignores auto-migration failure/skipping, then unconditionally opens/creates `omera.db`. [legacy_migration.rs:57](D:/dev/Omera/src-tauri/src/legacy_migration.rs:57) treats mere destination existence as `migrated` when there is no receipt. The Vue source has no calls to the new legacy migration status/preview/start commands.

With multiple discovered legacy databases, a conflict, or an unusable source, users see an empty Omera library and cannot choose/import a source through the intended migration workflow. The legacy data is preserved, but the normal retry path is lost because the empty destination now exists. Acceptance: persist pending/failed/choice-needed state separately from destination existence, expose source choice and retry/import in UI, and retain supported pre-1.0 import even after a new empty database is created.

### R06 — P1: Export privacy modes leak the prompt into text sidecars

**Reproduced.** [export.rs:271](D:/dev/Omera/crates/omera-scan/src/export.rs:271) suppresses `TextPrompt` only for `StripAll`; `StripPromptOnly` and `StripAllAiMetadata` still export the confidential positive prompt or raw metadata as `.txt`. The export dialog allows these combinations. Both policy probes fail with a synthetic embedded prompt.

The existing Rust export test at [export.rs:786](D:/dev/Omera/crates/omera-scan/src/export.rs:786) explicitly expects this leaking sidecar under `StripAllAiMetadata`, even while checking that the HTML showcase removes the prompt. Acceptance: apply one policy to embedded data, sidecars, showcase fields, and filenames; reject incompatible output combinations or produce sanitized results, and replace that contradictory test expectation.

### R07 — P1: Transformation metadata policies do not consistently control the actual output files

**Reproduced and code-confirmed.** [transform.rs:63](D:/dev/Omera/crates/omera-scan/src/transform.rs:63) performs raster transformation without implementing `metadata_policy`. A PNG-to-PNG resize with `KeepSupported` removes a supported embedded A1111 prompt. The normal export encoder likewise strips embedded metadata for all re-encoded outputs, even with `KeepAll` ([export.rs:205](D:/dev/Omera/crates/omera-scan/src/export.rs:205)); original-byte fast pass is the exception.

Conversely, managed import's fast-copy path can preserve original embedded metadata when only a stripping policy is selected, because the transform predicate ignores the policy. Its transformed `StripAi` path also copies original `.txt`/`.json` sidecars unfiltered at [commands.rs:864](D:/dev/Omera/src-tauri/src/commands.rs:864). Filtering only the database DTO does not sanitize the file users share. Acceptance: metadata preservation/stripping must be implemented at publication, unsupported preservation must be explicitly explained, and imported sidecars must follow the same policy. Verify exported/reimported files, not only in-memory metadata.

### R08 — P1: Batch transform archives or trashes the source before database persistence succeeds

**Reproduced.** [transform.rs:646](D:/dev/Omera/crates/omera-scan/src/transform.rs:646) applies original disposition before [transform.rs:679](D:/dev/Omera/crates/omera-scan/src/transform.rs:679) updates the file record. A synthetic trigger rejecting UPDATE causes a failed item whose indexed source is already archived and missing from its original path. There is no compensating rollback in that path.

Acceptance: publish/validate the derivative, persist an operation plan and destination record, then perform original disposition only after a durable commit; interrupted operations must be resumable. Preserve an archive with unique names and the source's sidecars. A failed persistence operation must leave the indexed original intact and the derivative clearly recoverable.

### R09 — P2: Successful transforms leave stale dimensions and private metadata in the library

**Reproduced.** [db.rs:688](D:/dev/Omera/crates/omera-storage/src/db.rs:688) changes path/container/size/mtime but does not update extracted metadata. A 64×32 source resized to 32×16 retains width 64 in the library; `StripAll` retains the original confidential prompt in its database record.

This breaks Inspector values and metadata-dependent searches/filters, and allows later exports to use stale prompt data. Acceptance: derive metadata from the published output under the selected policy, atomically update it with path/fingerprint, and invalidate affected thumbnails/embeddings where their source changed while retaining user organization.

### R10 — P2: The app can create AVIF files that its thumbnail decoder cannot read

**Reproduced.** [omera-scan/Cargo.toml:16](D:/dev/Omera/crates/omera-scan/Cargo.toml:16) enables AVIF encoding, but the built image decoder does not support AVIF. A generated AVIF derivative then fails the actual thumbnail service with `The image format Avif is not supported`. The transform's AVIF validation checks container branding instead of performing a supported decode.

Acceptance: enable and package a supported decoder across target platforms, or gate the format as export-only with clear import/preview limits. Test generation → import → thumbnail → preview → re-export using the same shipped build.

### R11 — P1: Upserting an existing file returns an unrelated file's ID

**Reproduced.** [db.rs:460](D:/dev/Omera/crates/omera-storage/src/db.rs:460) returns `last_insert_rowid()` after an UPSERT. Insert A, insert B, then upsert A: the returned ID is B's although A's row was updated. Pipeline harvesting uses that return value for `enqueue_cleanup`.

Acceptance: return the row ID from `INSERT ... RETURNING id` or a verified path lookup inside the same transaction. Test updates after intervening inserts, conflict updates inside bulk operations, and callers that attach albums/tags/cleanup jobs to returned IDs.

### R12 — P1: An incomplete filesystem walk is treated as proof that indexed files disappeared

**Code-confirmed.** [scanner.rs:493](D:/dev/Omera/crates/omera-scan/src/scanner.rs:493) discards WalkDir and metadata errors. [scanner.rs:339](D:/dev/Omera/crates/omera-scan/src/scanner.rs:339) deletes every existing path not observed by that walk. A temporarily unreadable subtree or interrupted network enumeration therefore looks like deletion; foreign-key cascades remove associated albums/tags/embeddings.

The OS permission failure was not injected on this host. Acceptance: record traversal coverage/errors, skip orphan deletion for incomplete regions, and surface a partial scan status. An unavailable subtree must retain its index and user organization until absence is positively established by a successful reconciliation.

### R13 — P1: Cloud object keys collide between library roots with the same basename

**Code-confirmed.** [cloud_sync.rs:378](D:/dev/Omera/src-tauri/src/cloud_sync.rs:378) uses the folder basename, and [cloud_sync.rs:411](D:/dev/Omera/src-tauri/src/cloud_sync.rs:411) constructs `prefix/basename/relative_path`. Two roots such as `D:\a\outputs` and `E:\b\outputs` both publish `outputs/image.png`. Different library files can overwrite the same remote object or be skipped as if already synchronized.

Acceptance: namespace objects with a stable storage-root UUID or another collision-proof identity, maintain a migration manifest for existing remote layouts, and reject duplicate remote keys before transfer. Preserve both assets in a two-root test for LocalPath, S3, and WebDAV.

### R14 — P1: Delta sync can miss changed files of the same byte length

**Code-confirmed.** FastFingerprint at [cloud_sync.rs:469](D:/dev/Omera/src-tauri/src/cloud_sync.rs:469) checks size only; collected mtime is discarded. WebDAV at [cloud_sync.rs:511](D:/dev/Omera/src-tauri/src/cloud_sync.rs:511) ignores the selected checksum strategy and ETag and also checks only size. S3 checksum mode treats a missing checksum header as a successful size match.

Two different equal-length files are therefore reported as synchronized. Acceptance: use a persisted fingerprint/content digest appropriate to each provider, do not treat unknown digests as verified, and test same-length edits, missing custom headers, remote changes, and dry-run result accuracy. This review did not run a real provider transfer.

### R15 — P2: Automatic backup configuration has no scheduler or usable control

**Code-confirmed.** `auto_backup_enabled` and `auto_backup_interval_days` occur in configuration models/defaults and settings load/save, but there is no execution path that schedules a snapshot from them. The Rust default enables this field, while frontend defaults disable it. The settings component holds and serializes these values but does not render an automatic-backup control. This is an incomplete configuration/feature contract, not an observed enabled GUI toggle that falsely reports protection.

Acceptance: implement a persisted scheduler with last successful backup, next due time, retry policy, startup catch-up, and visible failure state; or consistently mark scheduling unavailable and remove misleading enabled defaults. Verify a shortened test interval through actual snapshot creation and restart recovery. Enable a user-facing control only after scheduling works.

### R16 — P2: Real-time pipeline ingestion is configured but never triggered

**Code-confirmed.** [AddFolderModal.vue:270](D:/dev/Omera/src/components/AddFolderModal.vue:270) enables `auto_harvest` and promises continuous ingestion. The flag is stored but not consumed by a worker. [watcher.rs:101](D:/dev/Omera/src-tauri/src/watcher.rs:101) watches the managed destination `folder.path`, not pipeline `source_path`. The only Vue call to `harvest_pipeline_folder` is the manual Sidebar action.

Acceptance: observe the source with settled-write detection, bounded jobs, cancellation, and the safe publication/cleanup service from R02/R03; disabling the flag must stop automatic ingestion. Until then, label the workflow as manual and disable the real-time toggle.

### R17 — P2: Settings close and appear applied even when persistence fails

**Code-confirmed.** [SettingsModal.vue:591](D:/dev/Omera/src/components/SettingsModal.vue:591) logs save errors but still emits the saved state and closes. Theme/locale/cache settings are applied before persistence; [config.ts:197](D:/dev/Omera/src/utils/config.ts:197) mirrors failed writes into localStorage. A keyring, disk, or revision failure therefore produces divergent live/localStorage/backend values without an actionable error.

Reloading the latest config immediately before overlaying an older open form also weakens the intended revision conflict protection. Acceptance: retain the dialog and edits on failure, show the error, apply durable settings only after success or explicitly mark a temporary preview, and save against the revision loaded with the form.

### R18 — P2: Failed undo/redo permanently removes the recoverable command

**Reproduced.** [history.ts:45](D:/dev/Omera/src/utils/history.ts:45) pops before awaiting undo; redo does the same. A rejected IPC operation leaves the command in neither stack. Both rejection probes fail; existing tests cover only successful sequential operations.

The class also lacks an in-flight guard/serialization contract. Acceptance: transition stacks only after success, retain retryable commands on failure, serialize history operations, and test rapid shortcuts and overlapping commands with out-of-order completion. Define handling for partial backend success and unrelated Inspector edits.

### R19 — P2: Batch favorite/NSFW changes and their undo leave filters and counters stale

**Code-confirmed.** [App.vue:1541](D:/dev/Omera/src/App.vue:1541) updates flags in the loaded array without refreshing library counts or reapplying the active query; NSFW batch handling has the same shape. The `files` watcher only advances a gallery revision. In Favorites, batch unfavoriting can leave the item visible and the sidebar count unchanged; undo repeats the same state path.

Acceptance: use one mutation completion hook that refreshes counts and affected filtered/sorted pages while retaining valid selection/anchors. Cover favorite and NSFW removal from their own views, filter transitions, partial success, and undo/redo.

### R20 — P2: Statistics exposes model/sampler tabs that can never contain data

**Observed UI and code-confirmed.** [PromptStatsModal.vue:43](D:/dev/Omera/src/components/PromptStatsModal.vue:43) hardcodes `top_models: []` and `top_samplers: []`. Those pages always show an empty state, independent of the database. `total_analyzed` uses all indexed files rather than the population actually analyzed for prompt metadata.

Acceptance: implement the corresponding aggregate queries and a precisely defined analyzed count, or hide/disable those categories with a clear status. Distinguish unavailable data, a true empty category, and request failure. The old documented RPC argument mismatch is not a current finding: the present component sends `isNegative` correctly.

### R21 — P2: Table mode bypasses the NSFW blur preference

**Code-confirmed.** [FileList.vue:410](D:/dev/Omera/src/components/FileList.vue:410) renders normal thumbnails and has no `blurNsfw` prop, NSFW class, or reveal control. The same library's Grid/Waterfall path applies the setting at [VirtualGrid.vue:736](D:/dev/Omera/src/components/VirtualGrid.vue:736).

Acceptance: share a media privacy presentation policy across all gallery modes, including video previews and cull/selection previews. A mode change must not reveal a flagged image without the user's reveal choice. The live library contained no flagged files, so this was verified from code rather than a real sensitive-image UI test.

### R22 — P2: Menu and sidebar keyboard/accessibility support is incomplete

**Observed UI and code-confirmed.** Opening File and pressing Escape leaves its dropdown open. [MenuBar.vue:117](D:/dev/Omera/src/components/MenuBar.vue:117) installs only a click-outside listener and provides no menu keyboard/focus management or expanded state. Sidebar navigation is exposed as clickable list items and tag groups without an equivalent keyboard path. Gallery card checkboxes use the label “Select all” for selecting one image; Table row checkboxes are unnamed.

Acceptance: implement a coherent keyboard menu pattern, Escape/focus return, input-aware shortcut routing, keyboard-operable navigation/tags, and file-specific selection names. Verify with keyboard-only navigation and a screen reader. Existing modal infrastructure correctly closed the statistics dialog with Escape in the manual check; reuse its focus principles.

### R23 — P2: Advanced import transformation controls are silently ignored

**Code-confirmed.** [commands.rs:787](D:/dev/Omera/src-tauri/src/commands.rs:787) transforms only for a non-original format, max edge, or quality. Import UI emits scale percentage, alignment, and target-size fields, but selecting only one of these leaves the predicate false and performs a plain copy. Metadata-only policy suffers the related file-sanitization problem in R07.

Acceptance: define a single `requires_transform` predicate in the domain/service layer that covers every effective option. Test each option independently with Original format and verify resulting dimensions/size/metadata on disk.

### R24 — P2: Cloud sync memory and cancellation are bounded by file size, not a byte budget

**Code-confirmed.** [cloud_sync.rs:527](D:/dev/Omera/src-tauri/src/cloud_sync.rs:527) reads each entire media file into memory; concurrency permits up to 16 workers. Sixteen large videos can create many gigabytes of allocations. SHA mode may read once for hashing and again for upload. The limiter at [cloud_sync.rs:49](D:/dev/Omera/src-tauri/src/cloud_sync.rs:49) sleeps for the whole file's allowance while holding the shared limiter lock; cancellation is not checked during that sleep or the transfer.

Acceptance: stream hashing/upload in bounded chunks, enforce a global byte budget, make waits interruptible, and check cancellation between chunks. Test memory and cancellation latency with large synthetic files and a slow local endpoint. No memory peak or cancel-latency number was measured in this review.

### R25 — P2: Long transformations and export estimates retain the shared database mutex

**Code-confirmed.** [commands.rs:1793](D:/dev/Omera/src-tauri/src/commands.rs:1793) holds the shared `AppState.db` guard over the entire batch decode/encode/archive job. [commands.rs:1777](D:/dev/Omera/src-tauri/src/commands.rs:1777) holds it during an export estimate's image processing. Moving the operation into `spawn_blocking` does not release that mutex; library reads and mutations using it must wait behind the job.

Acceptance: snapshot bounded input under a short lock, run CPU/filesystem work without the shared guard, and persist short per-item transactions through a worker/connection contract. Measure gallery/search responsiveness while a slow codec batch is running. Avoid mechanically changing async signatures while preserving long lock ownership.

### R26 — P2: CI omits frontend tests and current test coverage misses safety failures

**Code-confirmed.** [.github/workflows/ci.yml:69](D:/dev/Omera/.github/workflows/ci.yml:69) builds the frontend but never runs `test:stack` or the IPC inventory check. Some tests inspect source strings or define separate mock action lists instead of mounting/using the component; these checks cannot establish real behavior. Four frontend suites are outside the main test script. The existing privacy test even encodes the leaking behavior described in R06 as correct.

Acceptance: run frontend behavior tests and generated-contract checks in CI, select all maintained suites consistently, and add real function/component fault tests for collisions, DB failures, failed IPC, partial operations, and restart recovery. Retain source-contract checks as supplemental safeguards rather than behavior evidence.

### R27 — P2: The checked-in release pipeline does not provision signed automatic updates

**Code-confirmed.** [update_verification.rs:4](D:/dev/Omera/src-tauri/src/update_verification.rs:4) requires an embedded Omera/Berry public key, and downloads require adjacent `.minisig` assets. The release workflow does not set either key variable or generate/upload these signature assets. Its macOS code signing/notarization does not supply the separate minisign update mechanism.

The application fails closed without update trust, which is appropriate security behavior. The gap is feature/release readiness: releases built solely by the checked-in workflow cannot complete that automatic-update path. Acceptance: provision signing and verification inputs, sign/publish every selected artifact, validate clean install → next-version update, and expose manual installation clearly when automatic updates are unavailable. External signing outside this workflow was not assessed.

### R28 — P2: Product claims and engineering contracts disagree with the running implementation

**Code-confirmed.** [README.md:57](D:/dev/Omera/README.md:57) and its cloud section advertise bidirectional sync and ETag/streaming semantics; the current sync path uploads and has the limitations in R13/R14/R24. [README.md:59](D:/dev/Omera/README.md:59) advertises full MySQL/PostgreSQL team storage while runtime uses `Mutex<Database>` with SQLite, config load/save forces SQLite, and those storage choices are disabled in settings. SQL export/traits are not a shipped remote storage runtime.

[ENGINEERING_HANDOFF.md:15](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md:15) says runtime identity/crate migration has not completed despite the current Omera identifiers/crates. API_CONTRACTS mixes old proposed-command language with newer implemented migration contracts. The IPC generator reports a stale inventory. Node/helper performance results also must not be presented as measured WebView paint/scroll latency. Acceptance: publish a current capability matrix, keep unavailable items explicit, regenerate IPC documentation and separately check DTOs, and record baseline/date/status for handoff tasks.

### R29 — P2: Localization and information presentation remain uneven

**Observed UI and code-confirmed.** [PromptStatsModal.vue:158](D:/dev/Omera/src/components/PromptStatsModal.vue:158), lines 274/317/320 contain “Refresh,” an English empty state, English guidance, and “Done” in the Chinese UI. New history action names/toasts and Sidebar error titles also use hardcoded English. Table mode gives filenames substantial fixed space while prompt/model columns require horizontal scrolling at the normal width; the long alphabetical tag cloud has no direct search/filter affordance.

Acceptance: place all user strings in locale files, include translated accessible labels/errors, and review terminology (“analyzed,” “synced,” “saved”) against actual state. For the layout, prioritize a searchable tag list and configurable/compact Table columns; these are UX improvements rather than proof of incorrect rendering. Keep fixed card widths and use column/gap changes for gallery whitespace.

### R30 — P2: Table mode hides collapsed stack members without an expansion affordance

**Observed UI and code-confirmed.** [App.vue:1769](D:/dev/Omera/src/App.vue:1769) collapses stack members in the shared `files` array during loading. [App.vue:2493](D:/dev/Omera/src/App.vue:2493) passes that array directly into FileList, which has no stack summary, member expansion event, or stack indicator. Switching modes only changes `viewMode`; it does not reload a flat list.

In the current 59-image library with one collapsed 12-image stack, Table shows 48 rows and its selection dock reports a total of 48. The other 11 members are inaccessible through a Table expansion control. Expanding in Grid first changes which members Table can show, so Table completeness depends on previous Grid interaction. Acceptance: load all matching rows for a flat Table, or implement explicit expandable stack rows with accurate image/row counts and equivalent keyboard access. Verify direct startup in Table, switching from collapsed/expanded Grid, search/sort, and paginated stacks. Basic Table Arrow Down selection worked in the manual check.

## Latent service defects and structural debt

### R31 — P2: Reusable managed-transform import accepts a linked destination

**Reproduced, latent.** [transform.rs:294](D:/dev/Omera/crates/omera-scan/src/transform.rs:294) resolves a destination ID but never validates `folder_type == managed`; the probe writes into a `link` folder and reports success. The registered `import_files_to_managed_vault` currently uses a different helper that does check its destination, so this is not an observed exploit of that IPC route.

Acceptance: enforce read-only ownership in the reusable service, consolidate duplicate import implementations, and test the same contract through every public entry point.

### R32 — P2: The registered cull command reports failed trash operations as successful

**Code-confirmed, latent in current Vue flow.** [commands.rs:4475](D:/dev/Omera/src-tauri/src/commands.rs:4475) discards trash errors, appends every candidate to `successfully_trashed_ids`, ignores database deletion errors, and increments the returned count. It also lacks a source ownership check. Current App.vue culling calls `trash_files` instead of this command, so the active UI follows another path.

Acceptance: remove an unused destructive IPC or delegate it to the common checked service, return per-item outcomes, and remove index rows only after confirmed filesystem success. Review the exact product contract for deliberate deletion of linked originals separately from automatic application-data cleanup.

### R33 — P2: Large orchestration modules and duplicated mutation services impede consistency

**Code-confirmed architectural debt.** `App.vue` is 3,217 lines, `commands.rs` 5,190, and `db.rs` 6,000. Size alone is not a bug, but import, pipeline harvesting/cleanup, cloud recovery, and transform paths implement overlapping safety rules in different places; several findings above follow directly from this duplication. Substantial business/filesystem logic remains in command adapters contrary to the repository's stated boundary.

Acceptance: refactor around tested services for publication, metadata policy, recovery, and operation receipts, with thin Tauri adapters and smaller UI controllers. Use measured, incremental changes rather than a whole-app rewrite; keep domain types free of I/O and append-only migrations intact.

## Product capability assessment

| User promise | Current evidence | PM action |
| --- | --- | --- |
| Browse/search a local library | Native modes render; virtualized helpers and existing tests pass | Preserve this core flow and verify larger datasets in the shipped WebView |
| Understand/reuse AIGC prompts | Broad parser suite passes; prompt stats positive tab works | Protect metadata across derivative/export/import workflows; correct incomplete stats |
| Organize/rate/favorite with undo | Basic history success path passes | Add error-safe history and query/counter consistency |
| Safely import and manage assets | Common file operation service has no-clobber behavior | Unify pipeline/transform/import safety before promoting automation |
| Privacy-safe sharing | Sanitized showcase fields exist | Fix sidecar and file metadata policy; verify all outputs |
| Back up and restore | Consistent VACUUM snapshot creation and local staged restore exist | Fix cloud restore; implement or disable scheduled backup |
| Incremental cloud synchronization | Upload workers/providers exist | Fix namespaces/fingerprints; describe upload-only behavior accurately |
| Automatic generator ingestion | Manual harvest exists | Implement source watching or label manual-only |
| Team database/collaboration | Traits/SQL export exist; shipped runtime remains SQLite | Mark remote runtime unavailable until full integration and validation |
| Automatic trusted updates | Verification rejects missing/tampered trust | Complete release provisioning and end-to-end installer/update checks |

The current GUI offers a clear three-pane library structure and convenient metadata access. More feature dialogs do not compensate for unreliable saved/synced/protected states. Favor a smaller, truthful capability surface with explicit failures and recovery actions.

## Recommended delivery order and acceptance

1. **Lead-owned data and trust corrections:** R01–R08, R11–R14, R27, plus migration artifact/cleanup validation. Follow the existing L1–L8 ownership boundaries. Establish one durable receipt/transaction lifecycle and use it across import, pipeline, transform, restore, and cleanup.
2. **Make existing UI promises complete:** R09/R10/R15 and R16–R23/R29/R30; independent bounded GUI/localization tasks may proceed against stable backend contracts. Do not expose cleanup UI until backend destination validation and separate confirmation are correct.
3. **Engineering and performance gates:** R24–R26/R28/R31–R33. Add safety tests first, then incremental refactoring and measured streaming/lock improvements. Bring docs and IPC inventory up to date.
4. **Release acceptance matrix:** all requested build/test/lint/format checks; actual old-version → Omera migration and Omera → next-version updates; corrupt/locked/ambiguous migration inputs; WAL restore and interrupted restore; same-name/equal-size imports; destination loss and replaced-source cleanup; all metadata policies/formats/sidecars; provider sync collisions/equal-size edits; failed undo/redo; keyboard/screen-reader/reduced-motion; Grid/Waterfall/Table at 960px and wide sizes, stack expand/collapse, rapid scrollbar dragging, and real 5k/50k libraries with warm/cold caches.

No credible schedule estimate follows from this review alone. First reproduce and fix the safety defects, then size the remaining work from the shared service contracts. Success should be measured by preserved data, verified recovery, honest feature states, and responsive real-library workflows rather than test counts or feature headings.
