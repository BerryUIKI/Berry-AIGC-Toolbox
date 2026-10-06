# GitHub publication record — 2026-10-05

Repository: [BerryUIKI/Omera](https://github.com/BerryUIKI/Omera). Reviewed runtime baseline: dev at bdac8fbcdbac5535075e091453b77c3afe518ba5. PR [#242](https://github.com/BerryUIKI/Omera/pull/242) was squash-merged into dev at a4ac24f37b178e12915edd96d9c6329ef0a31440; its three template/contribution files do not change runtime behavior.

There are **67 individual reports: 21 P1 and 46 P2**. Across the publication passes and the subsequent UX proposal, 64 new issues were created and 3 existing issues reopened. The earlier broad reports were narrowed to one independently reviewable topic each; related concerns received separate reports. Priority is a proposed triage severity, not a claim of exploitation or an executed destructive reproduction.

The aggregate tracker #265 is closed with state reason not_planned because the project requested one topic per issue. Closure does not indicate that any finding was fixed. Its original content is preserved locally in github-review-tracker-2026-10-05.md. Relevant synthetic production-function reproductions were copied to their individual reports.

## Individual reports

| Review ID | Priority | GitHub issue | Topic |
| --- | --- | --- | --- |
| R01 | P1 | [#98](https://github.com/BerryUIKI/Omera/issues/98) | Cloud restore bypasses the safe database restore lifecycle |
| R02 | P1 | [#232](https://github.com/BerryUIKI/Omera/issues/232) | Pipeline harvesting overwrites existing assets on filename collisions |
| R03 | P1 | [#233](https://github.com/BerryUIKI/Omera/issues/233) | Deferred pipeline cleanup does not revalidate the destination or source identity |
| R04 | P1 | [#234](https://github.com/BerryUIKI/Omera/issues/234) | Legacy configuration migration records success after persistence failure |
| R05 | P1 | [#235](https://github.com/BerryUIKI/Omera/issues/235) | Empty destination creation masks pending or failed legacy migration |
| R06 | P1 | [#236](https://github.com/BerryUIKI/Omera/issues/236) | Export privacy modes leak the prompt into text sidecars |
| R07 | P1 | [#237](https://github.com/BerryUIKI/Omera/issues/237) | KeepSupported transformation drops embedded generation metadata |
| R08 | P1 | [#238](https://github.com/BerryUIKI/Omera/issues/238) | Batch transform archives or trashes the source before database persistence succeeds |
| R09 | P2 | [#239](https://github.com/BerryUIKI/Omera/issues/239) | Batch transformation leaves database dimensions at their original values |
| R10 | P2 | [#240](https://github.com/BerryUIKI/Omera/issues/240) | The app can create AVIF files that its thumbnail decoder cannot read |
| R11 | P1 | [#241](https://github.com/BerryUIKI/Omera/issues/241) | Upserting an existing file returns an unrelated file's ID |
| R12 | P1 | [#243](https://github.com/BerryUIKI/Omera/issues/243) | An incomplete filesystem walk is treated as proof that indexed files disappeared |
| R13 | P1 | [#244](https://github.com/BerryUIKI/Omera/issues/244) | Cloud object keys collide between library roots with the same basename |
| R14 | P1 | [#245](https://github.com/BerryUIKI/Omera/issues/245) | FastFingerprint sync skips same-length content changes |
| R15 | P2 | [#246](https://github.com/BerryUIKI/Omera/issues/246) | Implement the configured automatic cloud backup scheduler |
| R16 | P2 | [#247](https://github.com/BerryUIKI/Omera/issues/247) | Real-time pipeline ingestion is configured but never triggered |
| R17 | P2 | [#248](https://github.com/BerryUIKI/Omera/issues/248) | Settings dialog reports success and closes after a failed save |
| R18 | P2 | [#249](https://github.com/BerryUIKI/Omera/issues/249) | Failed undo or redo removes the command from history |
| R19 | P2 | [#250](https://github.com/BerryUIKI/Omera/issues/250) | Batch favorite/NSFW changes and their undo leave filters and counters stale |
| R20 | P2 | [#251](https://github.com/BerryUIKI/Omera/issues/251) | Model and sampler statistics tabs receive hardcoded empty arrays |
| R21 | P2 | [#252](https://github.com/BerryUIKI/Omera/issues/252) | Table mode bypasses the NSFW blur preference |
| R22 | P2 | [#253](https://github.com/BerryUIKI/Omera/issues/253) | Application menus lack keyboard dismissal and focus navigation |
| R23 | P1 | [#254](https://github.com/BerryUIKI/Omera/issues/254) | Managed import bypasses processing for advanced or metadata-only transform settings |
| R24 | P2 | [#255](https://github.com/BerryUIKI/Omera/issues/255) | Cloud sync allocates whole files without a global memory budget |
| R25 | P2 | [#202](https://github.com/BerryUIKI/Omera/issues/202) | Batch transformations retain the shared database mutex during image processing |
| R26 | P2 | [#108](https://github.com/BerryUIKI/Omera/issues/108) | CI does not run the maintained frontend regression suites |
| R27 | P2 | [#256](https://github.com/BerryUIKI/Omera/issues/256) | The checked-in release pipeline does not provision signed automatic updates |
| R28 | P2 | [#257](https://github.com/BerryUIKI/Omera/issues/257) | README advertises bidirectional cloud sync that the runtime does not implement |
| R29 | P2 | [#258](https://github.com/BerryUIKI/Omera/issues/258) | Translate hardcoded statistics, history and Sidebar text into the selected locale |
| R30 | P2 | [#259](https://github.com/BerryUIKI/Omera/issues/259) | Table mode hides collapsed stack members without an expansion affordance |
| R31 | P2 | [#260](https://github.com/BerryUIKI/Omera/issues/260) | Reusable managed-transform import accepts a linked destination |
| R32 | P2 | [#261](https://github.com/BerryUIKI/Omera/issues/261) | Registered cull command counts failed trash operations as successful |
| R33 | P2 | [#262](https://github.com/BerryUIKI/Omera/issues/262) | Managed import duplicates publication logic across IPC and reusable services |
| R29-table | P2 | [#263](https://github.com/BerryUIKI/Omera/issues/263) | Make Table columns configurable and usable at normal window widths |
| R29-tags | P2 | [#264](https://github.com/BerryUIKI/Omera/issues/264) | Add search or filtering to the Sidebar tag list |
| R02-dedup | P1 | [#268](https://github.com/BerryUIKI/Omera/issues/268) | Managed import treats equal byte lengths as identical content |
| R04-source | P1 | [#269](https://github.com/BerryUIKI/Omera/issues/269) | Legacy migration reads configuration through a source-mutating loader |
| R04-cleanup | P1 | [#270](https://github.com/BerryUIKI/Omera/issues/270) | Legacy cleanup eligibility lacks configuration and model destination validation |
| R05-ui | P2 | [#271](https://github.com/BerryUIKI/Omera/issues/271) | Provide a legacy source-selection and retry workflow in the application |
| R07-export | P1 | [#272](https://github.com/BerryUIKI/Omera/issues/272) | KeepAll export strips embedded metadata during re-encoding |
| R07-sidecars | P1 | [#273](https://github.com/BerryUIKI/Omera/issues/273) | Managed import copies prompt-bearing sidecars despite stripping policies |
| R09-privacy | P1 | [#274](https://github.com/BerryUIKI/Omera/issues/274) | StripAll batch transformation retains the original private prompt in SQLite |
| R14-webdav | P1 | [#275](https://github.com/BerryUIKI/Omera/issues/275) | WebDAV delta sync ignores the selected checksum strategy |
| R14-s3 | P1 | [#276](https://github.com/BerryUIKI/Omera/issues/276) | S3 checksum mode treats a missing remote digest as a verified size match |
| R17-mirror | P2 | [#277](https://github.com/BerryUIKI/Omera/issues/277) | Failed configuration saves overwrite the localStorage mirror |
| R17-revision | P2 | [#278](https://github.com/BerryUIKI/Omera/issues/278) | Settings save reloads the latest revision before overwriting stale form values |
| R18-concurrency | P2 | [#279](https://github.com/BerryUIKI/Omera/issues/279) | ActionHistory permits overlapping undo transitions |
| R20-denominator | P2 | [#280](https://github.com/BerryUIKI/Omera/issues/280) | Prompt statistics counts all indexed files as analyzed |
| R34 | P2 | [#266](https://github.com/BerryUIKI/Omera/issues/266) | Adding a populated folder does not start its initial scan or populate the gallery |
| R10-discovery | P2 | [#267](https://github.com/BerryUIKI/Omera/issues/267) | Folder scanning ignores AVIF derivatives created by the application |
| R22-sidebar | P2 | [#281](https://github.com/BerryUIKI/Omera/issues/281) | Sidebar destinations and tags are not keyboard-operable |
| R22-labels | P2 | [#282](https://github.com/BerryUIKI/Omera/issues/282) | Individual image selection checkboxes have misleading or missing accessible names |
| R24-cancel | P2 | [#283](https://github.com/BerryUIKI/Omera/issues/283) | Cloud sync cancellation does not interrupt limiter waits or active transfers |
| R25-estimate | P2 | [#284](https://github.com/BerryUIKI/Omera/issues/284) | Export estimates retain the shared database mutex during decoding and encoding |
| R25-export | P2 | [#285](https://github.com/BerryUIKI/Omera/issues/285) | Batch export retains the shared database mutex throughout transcoding and output writes |
| R26-ipc-ci | P2 | [#286](https://github.com/BerryUIKI/Omera/issues/286) | CI does not check the generated IPC inventory for drift |
| R26-suite-selection | P2 | [#287](https://github.com/BerryUIKI/Omera/issues/287) | The frontend test entry point excludes four existing regression suites |
| R26-batch-test | P2 | [#288](https://github.com/BerryUIKI/Omera/issues/288) | Batch action tests assert copied behavior instead of mounting the production component |
| R28-remote-db | P2 | [#289](https://github.com/BerryUIKI/Omera/issues/289) | README claims shipped MySQL and PostgreSQL storage while runtime is SQLite-only |
| R28-handoff | P2 | [#290](https://github.com/BerryUIKI/Omera/issues/290) | Engineering handoff reports an outdated identity migration status |
| R28-api | P2 | [#291](https://github.com/BerryUIKI/Omera/issues/291) | API contracts contradict the implementation status of legacy migration commands |
| R28-inventory | P2 | [#292](https://github.com/BerryUIKI/Omera/issues/292) | The checked-in IPC reference does not match the generated command inventory |
| R33-pipeline | P2 | [#293](https://github.com/BerryUIKI/Omera/issues/293) | Extract pipeline harvesting and cleanup business logic from Tauri command adapters |
| R26-format | P2 | [#294](https://github.com/BerryUIKI/Omera/issues/294) | Existing Rust formatting drift fails CI on every platform |
| R35 | P2 | [#295](https://github.com/BerryUIKI/Omera/issues/295) | Select All silently selects only the loaded page in a larger gallery |
| R36 | P2 | [#296](https://github.com/BerryUIKI/Omera/issues/296) | Negative prompt exclusions trigger false-positive NSFW classification |
| R37 | P2 | [#297](https://github.com/BerryUIKI/Omera/issues/297) | Add a main-toolbar toggle for sensitive-content masking |

## Templates and verification

PR #242 contains bug_report.yml, feature_request.yml and CONTRIBUTING.md reporting guidance. YAML metadata, unique field IDs, types, labels and whitespace checks passed. The existing Rust formatting drift failed the three platform CI checks; this separate integration-branch defect is tracked in #294. The documentation-only PR was merged using the repository's existing administrator permission, as explicitly requested by the user.

The repository default branch remains main. GitHub issue forms become available in the normal issue chooser when the merged template files reach that branch through the project's release process.

All 67 report titles, complete bodies, exact label sets and OPEN states were verified against the prepared English content. The durable manifest is github-atomic-publication-2026-10-05.jsonl; the verification summary is github-atomic-verification-summary-2026-10-05.json. Issue bodies provide baseline permalinks, evidence strength, reproduction/validation instructions and focused acceptance criteria. Existing discussions/comments were retained; prior issue bodies are archived in the original local snapshots, while the active bodies now state the narrower scope. R37 is a user-requested UX proposal supported by inspection of the current settings and gallery toolbar; it does not describe an implemented change.

No real user images, prompts, source filenames or private inventories were uploaded to GitHub. Review artifacts remain local and untracked. The unrelated src-tauri/Cargo.toml working-tree change was preserved.

## Real-sample follow-up

The real-image scan, copied-media operations, integrity checks, new findings #266/#267/#295/#296 and remaining verification limits are documented in REAL_SAMPLE_TESTS_2026-10-05.md. The development application was run from the root on dev and remains available for follow-up testing.

