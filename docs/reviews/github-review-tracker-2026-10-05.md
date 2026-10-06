<!-- omera-review-2026-10-05:tracker -->

## Purpose and baseline

Track the actionable findings from the product and engineering review of Omera 0.4.3, `dev` at [`bdac8fb`](https://github.com/BerryUIKI/Omera/commit/bdac8fbcdbac5535075e091453b77c3afe518ba5), reviewed on Windows on 2026-10-05. The native application was launched from the repository root using `pnpm run tauri dev`.

There are **35 actionable items: 12 P1 and 23 P2**. The review's 33 finding groups were retained; localization, Table column usability, and tag-list search were split into separate issues. **32 new reports were created and 3 existing issues were reopened** after checking the open/closed issue history. This tracking issue is additional to those 35 items.

P1 denotes high-impact data/privacy/recovery defects to address before broad release. P2 denotes important correctness, feature-completeness, accessibility, performance, documentation or architecture work. Priority is a triage proposal. Individual issues distinguish direct reproduction, native UI observation, traced code paths and latent service defects; an inspection-derived validation scenario is not a claim of an executed destructive or remote reproduction.

## P1: data safety, privacy, migration and sync correctness

- [ ] #98 — Cloud restore bypasses the safe database restore lifecycle (reopened with current-baseline evidence)
- [ ] #232 — Pipeline harvesting can overwrite a different existing asset
- [ ] #233 — Deferred pipeline cleanup does not revalidate the destination or source identity
- [ ] #234 — Legacy configuration migration can fail and still be recorded as successful
- [ ] #235 — Ambiguous or failed legacy migration is hidden behind a newly created empty library
- [ ] #236 — Export privacy modes leak the prompt into text sidecars
- [ ] #237 — Transformation metadata policies do not consistently control the actual output files
- [ ] #238 — Batch transform archives or trashes the source before database persistence succeeds
- [ ] #241 — Upserting an existing file returns an unrelated file's ID
- [ ] #243 — An incomplete filesystem walk is treated as proof that indexed files disappeared
- [ ] #244 — Cloud object keys collide between library roots with the same basename
- [ ] #245 — Delta sync can miss changed files of the same byte length

## P2: correctness, usability, completeness and engineering quality

- [ ] #239 — Successful transforms leave stale dimensions and private metadata in the library
- [ ] #240 — The app can create AVIF files that its thumbnail decoder cannot read
- [ ] #246 — Automatic backup configuration has no scheduler or usable control
- [ ] #247 — Real-time pipeline ingestion is configured but never triggered
- [ ] #248 — Settings close and appear applied even when persistence fails
- [ ] #249 — Failed undo/redo permanently removes the recoverable command
- [ ] #250 — Batch favorite/NSFW changes and their undo leave filters and counters stale
- [ ] #251 — Statistics exposes model/sampler tabs that can never contain data
- [ ] #252 — Table mode bypasses the NSFW blur preference
- [ ] #253 — Menu and sidebar keyboard/accessibility support is incomplete
- [ ] #254 — Advanced import transformation controls are silently ignored
- [ ] #255 — Cloud sync memory and cancellation are bounded by file size, not a byte budget
- [ ] #202 — Long transformations and export estimates retain the shared database mutex (reopened with current-baseline evidence)
- [ ] #108 — CI omits frontend tests and current test coverage misses safety failures (reopened with current-baseline evidence)
- [ ] #256 — The checked-in release pipeline does not provision signed automatic updates
- [ ] #257 — Product claims and engineering contracts disagree with the running implementation
- [ ] #258 — Translate hardcoded statistics, history and Sidebar text into the selected locale
- [ ] #259 — Table mode hides collapsed stack members without an expansion affordance
- [ ] #260 — Reusable managed-transform import accepts a linked destination
- [ ] #261 — The registered cull command reports failed trash operations as successful
- [ ] #262 — Large orchestration modules and duplicated mutation services impede consistency
- [ ] #263 — Make Table columns configurable and usable at normal window widths
- [ ] #264 — Add search or filtering to the Sidebar tag list

## Dependencies and delivery boundaries

- Correct publication/content identity #232 and returned row IDs #241 before relying on deferred cleanup #233; qualify those contracts before adding automatic ingestion #247.
- Establish a shared metadata policy #237 and privacy outputs #236 together with durable transform disposition #238 and catalog refresh #239.
- Qualify artifact-specific migration #234 and visible source selection/retry #235 before exposing legacy cleanup. Preserve source data; cleanup still requires validated destination data and a separate application decision.
- Use the regression cases to drive incremental service extraction #262 and shorten shared database locks #202.
- Complete the checked-in update signing pipeline #256 before treating automatic updates as release-ready.

High-impact persistence, identity, cleanup, security and release implementation remains lead-owned under [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md). Fix branches and PRs target `dev`; applied migrations remain append-only. This review does not authorize automatic legacy media/vault/shared-directory cleanup.

## Baseline verification

| Check | Result | Scope |
| --- | --- | --- |
| `pnpm run build` | PASS | Typecheck and production build |
| `pnpm run test:stack` | PASS: 154 tests | Currently selected frontend/helper suites |
| Four maintained frontend suites outside that script | PASS: 7 tests | Auto-tag mounting/actions, removal confirmation and storage-backend coverage |
| `cargo test --workspace` | PASS: 244 unit tests | Current workspace suite |
| `cargo clippy --workspace -- -D warnings` | PASS | CI's additional `--all-targets` variant was not separately run |
| `cargo fmt --check` | FAIL | Formatting differences in `src-tauri/src/commands.rs` near lines 2770, 2819 and 2838 |
| `node scripts/generate-ipc-reference.mjs --check` | FAIL | Stale inventory; tracked with documentation/contracts #257 |
| Additional production-function acceptance probes | FAIL: 10 Rust + 2 TypeScript scenarios | Expected safety/correctness behavior is violated; sources below |
| Native development app | Inspected | Grid, Waterfall, Table, stacks, basic Table navigation, statistics, menus, normal/maximized windows |

The Vite build warns about a 580.67 kB entry chunk (195.63 kB gzip). This is an optimization signal, **not a measured startup regression**, and was not reported as a separate bug.

## Release acceptance still required

- [ ] Resolve the linked safety/correctness items and repair the formatting baseline; rerun the relevant required checks, including generated inventory and separate DTO compatibility.
- [ ] Exercise interrupted/WAL restore, nonzero-revision and ambiguous legacy migration, destination-loss/replaced-source cleanup, equal-size collisions, metadata policies and rejected history operations with synthetic fixtures.
- [ ] Validate actual S3/WebDAV transfers and update installers/signatures on supported platforms. These were not exercised in this review.
- [ ] Verify all gallery modes at narrow/wide sizes, rapid scrollbar dragging, stack expand/collapse, keyboard/screen-reader and reduced-motion behavior.
- [ ] Measure real WebView latency/memory with large libraries and background work. Node/helper timings do not establish 50,000-item WebView performance.

Keep an issue open until its own acceptance criteria have implementation and validation evidence. Passing the existing suites alone does not resolve the additional failures below.

## English issue forms

[PR #242](https://github.com/BerryUIKI/Omera/pull/242) adds Bug report and Feature request forms plus contribution guidance. It targets `dev` and is a draft. Both forms passed strict YAML parsing and structural checks. GitHub reads the chooser from the default branch, currently `main`, so activation follows the normal release promotion; see [GitHub documentation](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/configuring-issue-templates-for-your-repository).

## Minimal acceptance reproduction package

The following sources call actual repository functions/classes with **synthetic temporary fixtures**. The Rust harness has independent dependency resolution and did not use the application's root lockfile in this review. Its observed failures agree with the inspected production source, but dependency-sensitive behavior should also be verified in the shipped build. No real media/database restore or live provider credentials are needed.

Create the files at the paths indicated in a checkout of the baseline, then run from the repository root:

```powershell
cargo test --manifest-path docs/reviews/repro-2026-10-05/Cargo.toml --target-dir target probes:: -- --test-threads=1
pnpm exec tsx --test docs/reviews/repro-2026-10-05/history-probes.test.mjs
```

The acceptance tests are intentionally red on this baseline. On the review host, cloud restore fails before replacement with Windows error 1224 (a user-mapped section is open); corruption on a platform allowing replacement was not demonstrated. The managed-import helper probe is latent: the current registered import command uses a different helper with a destination guard.

<details>
<summary>Cargo.toml</summary>

Save as `docs/reviews/repro-2026-10-05/Cargo.toml`:

```toml
[package]
name = "omera-review-probes"
version = "0.0.0"
edition = "2021"

[workspace]

[dependencies]
omera-domain = { path = "../../../crates/omera-domain" }
omera-storage = { path = "../../../crates/omera-storage" }
omera-metadata = { path = "../../../crates/omera-metadata" }
omera-scan = { path = "../../../crates/omera-scan" }
image = { version = "0.25", default-features = false, features = ["png", "jpeg", "webp", "avif", "rayon"] }
tempfile = "3"
png = "0.18"
base64 = "0.22"
sha2 = "0.10"
hex = "0.4"
zip = { version = "2.2", default-features = false, features = ["deflate"] }
serde_json = "1"
ureq = { version = "2.10", default-features = false, features = ["tls"] }
```

</details>

<details>
<summary>Rust production-function probes and synthetic fixtures</summary>

Save as `docs/reviews/repro-2026-10-05/src/lib.rs`:

```rust
#[path = "../../../../src-tauri/src/cloud_backup.rs"]
mod cloud_backup;

#[cfg(test)]
mod probes {
    use omera_domain::*;
    use omera_storage::Database;
    use std::{fs, path::Path};

    fn make_png(path: &Path) {
        let writer = fs::File::create(path).unwrap();
        let mut encoder = png::Encoder::new(writer, 64, 32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.add_text_chunk("parameters".into(), "confidential prompt\nNegative prompt: private negative\nSteps: 20, Sampler: Euler, CFG scale: 7, Seed: 123, Size: 64x32, Model: private-model".into()).unwrap();
        encoder.write_header().unwrap().write_image_data(&vec![120; 64 * 32 * 3]).unwrap();
    }

    fn make_file(path: &Path, folder_id: i64) -> ImageFile {
        ImageFile {
            id: None, folder_id, path: path.to_string_lossy().into_owned(),
            size_bytes: fs::metadata(path).unwrap().len(), modified_at: 1,
            container: Container::Png, metadata: omera_metadata::extract_metadata(Container::Png, path),
            rating: Some(4), aesthetic_score: None, is_favorite: true, is_nsfw: false,
            stack_id: None, stack_order: 0,
        }
    }

    fn export_options(privacy: MetadataPrivacyMode) -> ExportOptions {
        ExportOptions {
            file_ids: vec![], format: ExportFormat::Png, quality: 85, privacy,
            sidecar: ExportSidecar::TextPrompt, filename_template: "{name}".into(),
            destination_path: "unused".into(), as_zip: false, max_edge: None,
            export_html_showcase: false, html_title: None,
        }
    }

    #[test]
    fn stripping_prompt_must_not_emit_prompt_text_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let file = make_file(&source, 1);
        for policy in [MetadataPrivacyMode::StripPromptOnly, MetadataPrivacyMode::StripAllAiMetadata] {
            let output = omera_scan::export::process_single_image(&file, &export_options(policy), 0).unwrap();
            assert!(output.sidecar.is_none(), "Privacy policy {policy:?} emitted a confidential prompt sidecar");
        }
    }

    #[test]
    fn strip_ai_metadata_must_not_emit_prompt_text_sidecar() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let output = omera_scan::export::process_single_image(&make_file(&source, 1), &export_options(MetadataPrivacyMode::StripAllAiMetadata), 0).unwrap();
        assert!(output.sidecar.is_none(), "StripAllAiMetadata emitted a confidential prompt sidecar");
    }

    #[test]
    fn upsert_existing_path_must_return_its_actual_id() {
        let dir = tempfile::tempdir().unwrap();
        let source_a = dir.path().join("a.png"); make_png(&source_a);
        let source_b = dir.path().join("b.png"); make_png(&source_b);
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder(dir.path().to_str().unwrap()).unwrap();
        let file_a = make_file(&source_a, folder.id);
        let id_a = db.upsert_file(&file_a).unwrap();
        let id_b = db.upsert_file(&make_file(&source_b, folder.id)).unwrap();
        assert_ne!(id_a, id_b);
        assert_eq!(db.upsert_file(&file_a).unwrap(), id_a, "Updating a.png returns the unrelated b.png ID");
    }

    #[test]
    fn cloud_restore_must_survive_active_wal_connection() {
        let dir = tempfile::tempdir().unwrap();
        let active_path = dir.path().join("active.db");
        let active = Database::connect(&active_path).unwrap();
        active.add_folder("active-before-checkpoint").unwrap();
        active.connection().execute_batch("PRAGMA wal_checkpoint(TRUNCATE);").unwrap();
        active.add_folder("active-in-wal").unwrap();
        let snapshot_db = Database::connect_in_memory().unwrap();
        snapshot_db.add_folder("restored-snapshot").unwrap();
        let cloud = dir.path().join("cloud");
        let config = CloudBackupConfig { local_path: Some(cloud.to_string_lossy().into_owned()), ..Default::default() };
        let created = crate::cloud_backup::create_cloud_snapshot(&snapshot_db, &config, dir.path(), None).unwrap();
        let filename = created.snapshot.unwrap().filename;
        let restored = crate::cloud_backup::restore_cloud_snapshot(&active_path, &config, &filename).unwrap();
        assert!(restored.success);
        drop(active);
        let reopened = Database::connect(&active_path).unwrap();
        let folders = reopened.list_folders().unwrap();
        assert_eq!(folders.len(), 1, "Cloud restore reported success, but active WAL overwrote the restored data: {folders:?}");
        assert_eq!(folders[0].path, "restored-snapshot");
    }

    #[test]
    fn keep_supported_transform_must_preserve_embedded_prompt() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let spec = TransformSpec { format: TransformFormat::Png, max_edge: Some(32), ..Default::default() };
        let output = omera_scan::transform_file_staged(&source, &dir.path().join("stage"), &spec).unwrap();
        let parsed = omera_metadata::extract_metadata(Container::Png, &output);
        assert_eq!(parsed.and_then(|m| m.prompt), Some("confidential prompt".into()));
    }

    #[test]
    fn exported_avif_must_be_decodable_for_library_thumbnail() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let spec = TransformSpec { format: TransformFormat::Avif, ..Default::default() };
        let output = omera_scan::transform_file_staged(&source, &dir.path().join("stage"), &spec).unwrap();
        let result = omera_scan::thumbnail::generate_thumbnail(&output, &dir.path().join("thumb.webp"), 32);
        assert!(result.is_ok(), "Created AVIF fails library thumbnail decode: {result:?}");
    }

    #[test]
    fn managed_transform_import_must_reject_linked_destination() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let dest = dir.path().join("external"); fs::create_dir(&dest).unwrap();
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder(dest.to_str().unwrap()).unwrap();
        let request = ImportTransformRequest { managed_destination_id: folder.id,
            source_paths: vec![source.to_str().unwrap().into()], spec: TransformSpec::default(),
            source_disposition: ImportSourceDisposition::Keep };
        let result = omera_scan::execute_managed_import_transform(&db, &request, None::<fn(usize, usize, &str)>);
        assert!(result.is_err(), "Managed import writes successfully into a read-only linked folder: {result:?}");
    }

    #[test]
    fn archive_must_wait_until_database_update_succeeds() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder_with_mode(dir.path().to_str().unwrap(), "managed", None, None, None, false).unwrap();
        let file = make_file(&source, folder.id);
        let id = db.upsert_file(&file).unwrap();
        db.connection().execute_batch("CREATE TRIGGER review_fail_update BEFORE UPDATE ON files BEGIN SELECT RAISE(FAIL, 'review simulated database write failure'); END;").unwrap();
        let request = LibraryTransformRequest { file_ids: vec![id], spec: TransformSpec { format: TransformFormat::Jpeg, ..Default::default() }, original_disposition: OriginalDisposition::Archive };
        let result = omera_scan::execute_library_batch_transform(&db, &request, None::<fn(usize, usize, &str)>).unwrap();
        assert_eq!(result.failed, 1);
        assert!(source.exists(), "Source was archived before a failed DB update; indexed source path is now missing");
    }

    #[test]
    fn transformed_database_dimensions_must_match_derivative() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder_with_mode(dir.path().to_str().unwrap(), "managed", None, None, None, false).unwrap();
        let id = db.upsert_file(&make_file(&source, folder.id)).unwrap();
        let request = LibraryTransformRequest { file_ids: vec![id], spec: TransformSpec { format: TransformFormat::Jpeg, max_edge: Some(32), ..Default::default() }, original_disposition: OriginalDisposition::Keep };
        let result = omera_scan::execute_library_batch_transform(&db, &request, None::<fn(usize, usize, &str)>).unwrap();
        assert_eq!(result.succeeded, 1);
        let file = db.get_file_by_id(id).unwrap().unwrap();
        assert_eq!(file.metadata.unwrap().width, Some(32), "Inspector retains original dimensions after successful resize");
    }

    #[test]
    fn strip_all_batch_transform_must_not_retain_prompt_in_library() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("source.png"); make_png(&source);
        let db = Database::connect_in_memory().unwrap();
        let folder = db.add_folder_with_mode(dir.path().to_str().unwrap(), "managed", None, None, None, false).unwrap();
        let id = db.upsert_file(&make_file(&source, folder.id)).unwrap();
        let request = LibraryTransformRequest { file_ids: vec![id], spec: TransformSpec { format: TransformFormat::Jpeg, metadata_policy: TransformMetadataPolicy::StripAll, ..Default::default() }, original_disposition: OriginalDisposition::Keep };
        let result = omera_scan::execute_library_batch_transform(&db, &request, None::<fn(usize, usize, &str)>).unwrap();
        assert_eq!(result.succeeded, 1);
        let metadata = db.get_file_by_id(id).unwrap().unwrap().metadata;
        assert!(metadata.as_ref().and_then(|m| m.prompt.as_ref()).is_none(), "strip_all transform still exposes original private prompt in library");
    }
}
```

</details>

<details>
<summary>TypeScript failed undo/redo probes</summary>

Save as `docs/reviews/repro-2026-10-05/history-probes.test.mjs`:

```js
import test from 'node:test';
import assert from 'node:assert/strict';
import { ActionHistory } from '../../../src/utils/history.ts';

test('failed undo must retain its command for recovery or retry', async () => {
  const history = new ActionHistory();
  await history.execute({ name: 'set rating', execute: async () => {}, undo: async () => { throw new Error('simulated IPC rejection'); } });
  await assert.rejects(history.undo());
  assert.equal(history.canUndo(), true, 'Failed undo removed the only recoverable command');
});

test('failed redo must retain its command for recovery or retry', async () => {
  const history = new ActionHistory();
  await history.execute({ name: 'set rating', execute: async () => {}, undo: async () => {}, redo: async () => { throw new Error('simulated IPC rejection'); } });
  await history.undo();
  await assert.rejects(history.redo());
  assert.equal(history.canRedo(), true, 'Failed redo removed the only recoverable command');
});
```

</details>

