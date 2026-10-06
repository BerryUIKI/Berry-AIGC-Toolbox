# Real-sample follow-up — 2026-10-05

The root checkout is on dev. Issue-template PR #242 was merged first, at a4ac24f37b178e12915edd96d9c6329ef0a31440, and the native application was launched with pnpm run tauri dev. This merge changes contribution documentation/templates only; the reviewed runtime source remains the bdac8fb baseline. No application bug fix was made during this testing pass.

## Dataset and source preservation

The user authorized the AI-generated images under D:\AIGC for remaining tests. Read-only inventory found 1,276 PNG images across four subdirectories, 3,093,281,849 bytes in total. All have A1111 parameters; 1,275 are RGB and one is RGBA. Largest image area is 2,073,600 pixels. There were no header-read errors.

Twenty-five representative images were copied to a temporary fixture directory, covering all four subdirectories and byte-size/pixel-count extremes. Destructive/transform operations targeted copies and temporary managed destinations only. Exact SHA-256 comparison before copying, during testing, and after testing confirmed that all 25 corresponding originals and all 25 fixture copies remained unchanged.

The application now indexes the authorized root as a read-only linked folder. Its 1,276 images are additional to the existing 59-item library, producing 1,335 indexed items. Tests did not execute trash, archive, cleanup or destructive actions against the original dataset. Temporary selection was cleared. No user image or prompt was published to GitHub.

Private inventory/fixture manifests contain source paths and remain local. Public reports use synthetic reproductions and sanitized counts.

## Production-service results

The harness imports the repository's actual crates and cloud-backup module. Database, staging, managed-import, ZIP and local backup destinations are isolated temporary directories. These measurements use the debug build and local filesystem cache; they are not WebView frame-time or general performance claims.

| Scenario | Result | Evidence |
| --- | --- | --- |
| Scan all authorized images into isolated SQLite | PASS: found/added 1,276; failed 0; prompt extraction 1,276/1,276 | 9,536 ms; 92 progress events |
| Repeat unchanged scan | PASS: unchanged 1,276; added/updated/removed/failed 0 | 14 ms in this warm-cache run |
| Decode and thumbnail representative copies | PASS: 25/25 | 256-pixel thumbnail target |
| Original format, KeepAll, no resizing | PASS: byte-exact 25/25 | Compared output bytes with input copies |
| PNG resize with KeepSupported | FAIL: embedded prompts lost 25/25 | [#237](https://github.com/BerryUIKI/Omera/issues/237) |
| Re-encoded PNG export with KeepAll | FAIL: embedded prompts lost 25/25 | [#272](https://github.com/BerryUIKI/Omera/issues/272) |
| StripPromptOnly with TextPrompt sidecar | FAIL: nonempty prompt sidecar 25/25 | [#236](https://github.com/BerryUIKI/Omera/issues/236) |
| PNG/JPEG/WebP 128-edge transformation on a real copy | PASS: all derivatives reopen at 96×128 | Tested one representative source, not every format on every image |
| Managed-transform service, rename collision | PASS: two imports succeed under distinct paths | Reusable service; not a full GUI import test |
| Managed-transform service, skip collision | PASS: one skipped receipt | Reusable service |
| ZIP batch export | PASS: 2 exported, 0 failed; both archived PNGs decode | Temporary managed records |
| Local-path snapshot and closed-connection restore | PASS: 2 rows restored; integrity_check=ok | Does not invalidate the active-connection restore defect #98 |
| Create AVIF, then generate thumbnail | FAIL: creation succeeds, thumbnail decoding fails | [#240](https://github.com/BerryUIKI/Omera/issues/240) |
| Scan app-generated AVIF-only directory | FAIL: found=0, added=0 | [#267](https://github.com/BerryUIKI/Omera/issues/267) |
| Final source/copy hash comparison | PASS: 25/25 originals, 25/25 copies unchanged | real-source-hash-verification-2026-10-05.json |

Commands from the repository root:

```powershell
cargo run --manifest-path docs/reviews/repro-2026-10-05/Cargo.toml --target-dir target --bin real_samples -- docs/reviews/real-test-fixtures-2026-10-05.json D:\AIGC docs/reviews/real-sample-test-results-2026-10-05.json
cargo run --manifest-path docs/reviews/repro-2026-10-05/Cargo.toml --target-dir target --bin real_operations -- docs/reviews/real-test-fixtures-2026-10-05.json docs/reviews/real-operation-test-results-2026-10-05.json
python docs/reviews/verify-real-source-hashes.py
```

The library module's unused cloud-function warnings are expected in this small standalone harness. The production crate dependencies and source are used directly.

## Native UI observations and additional findings

- Adding the populated linked folder initially showed zero images and an empty-folder/retry state. Source inspection confirms that folder registration and onFolderAdded do not start an initial scan. A watcher alone does not discover already-existing unchanged files. User independently reported this same experience. [#266](https://github.com/BerryUIKI/Omera/issues/266) tracks this single workflow, including background initial indexing, progress, automatic results/count refresh and visible retry.
- Grid and Waterfall rendered the real library at the normal approximately 1280×850 viewport. Table was observed at normal and maximized 2560×1392 sizes. Table displayed unblurred thumbnails while Waterfall applied the NSFW preference; this adds native evidence to [#252](https://github.com/BerryUIKI/Omera/issues/252).
- Ctrl+A initially selected 400 and displayed 400/400, despite the folder containing 1,276 indexed files. onSelectAll uses only the loaded files array. [#295](https://github.com/BerryUIKI/Omera/issues/295) tracks the selection-scope defect.
- A rapid Grid scrollbar drag moved to the end of the loaded range, then a following page appended and visible thumbnails settled. Ctrl+A now selected 800/800, confirming that its scope grows with loaded pages rather than selecting the full 1,276-result population. Selection was cleared afterwards. This was a functional observation; no frame-latency or memory trace was collected.
- The real library's high automatic NSFW count prompted a targeted synthetic test. A solid-color landscape PNG with adult terms only in its negative prompt is classified NSFW because the classifier scans raw parameter text. The production extractor/classifier probe fails deterministically. [#296](https://github.com/BerryUIKI/Omera/issues/296) tracks this false-positive mechanism; the test does not establish that every flagged real image is a false positive.
- A controlled barrier test against the production ActionHistory class records two simultaneous undo callbacks. [#279](https://github.com/BerryUIKI/Omera/issues/279) tracks this separately from rejected-transition history loss in #249.

## Verification limits

The minimum-width resize attempt did not establish a verified 960-pixel viewport. This run does not certify all gallery modes at minimum width, reduced-motion behavior, 10k/50k native rendering, actual WebView retained memory/frame timing, or long stress stability. Normal/wide observations are limited to those explicitly described above.

No live S3/WebDAV credentials or provider environment were supplied; their findings remain code-confirmed and require isolated protocol fixtures/live integration acceptance. Installer, signed-update delivery, macOS and Linux runtime behavior were not exercised on this Windows development host. Real legacy application-data cleanup and real-library destructive failure injection were not run; synthetic tests and source inspection were used instead.

Existing baseline build/frontend/Rust checks are recorded in 2026-10-05_COMPREHENSIVE_REVIEW.md. The new classification and history concurrency acceptance probes intentionally fail; these failures are evidence of defects, not passing regressions. IPC inventory --check still fails, and existing formatting drift is tracked in #294.

The complete one-topic-per-issue index is GITHUB_PUBLICATION_2026-10-05.md. All 66 issue titles, bodies, labels and open states were checked exactly after publication. Tracker #265 is closed administratively; its findings were not marked fixed.
