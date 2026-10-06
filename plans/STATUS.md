# Remediation execution status

Initialized October 5, 2026. No issue was revalidated or fixed by the document split. This is a manually maintained coordinator file; regeneration must not replace it. Frozen review evidence is separate.

Allowed states: `unverified`, `reproduced`, `in-progress`, `ready-for-review`, `verified`, `blocked`, `deferred`. Only `verified` with integrated-candidate acceptance counts as complete; deferred/waived topics remain unresolved.

Record current baseline/HEAD, dirty state, assigned owner, active shared-file ownership and prerequisite handoff links below. Each progress row needs an exact candidate/evidence link when its status changes.

**Current execution baseline:** pending assignment.  
**Active owners/shared-file slots:** none assigned by this split.

| Task | State | Owner / branch | Integrated candidate / acceptance evidence / limits |
| --- | --- | --- | --- |
| [R26-suite-selection](tasks/R26-suite-selection.md) | verified | QA engineer / codex/r26-suite-selection | Integrated in PR #299 (commit cb5cbaa); all 39 test suites executed (161 passing tests) via scripts/run-tests.mjs; CI passed; closed issue #287 |
| [R26-batch-test](tasks/R26-batch-test.md) | verified | UI QA engineer / codex/r26-batch-test | Integrated in PR #300 (commit a0703c8); mounted production BatchActionBar.vue with real emitted events & selection testing; CI passed; closed issue #288 |
| [R26-format](tasks/R26-format.md) | verified | Rust engineer / codex/r26-format-drift | Integrated in PR #298 (commit 0aee4d5); local cargo fmt --check passed and multi-platform CI passed; closed issue #294 |
| [R28-handoff](tasks/R28-handoff.md) | verified | Documentation engineer / codex/r28-handoff | Integrated in PR #304 (commit ead5498); reconciled identity migration status in docs/ENGINEERING_HANDOFF.md; CI passed; closed issue #290 |
| [R28-api](tasks/R28-api.md) | verified | Lead contract/documentation engineer / codex/r28-api | Integrated in PR #305 (commit a4a06a7); reconciled migration IPC status in docs/API_CONTRACTS.md and added regression tests for DTO compatibility; CI passed; closed issue #291 |
| [R28-inventory](tasks/R28-inventory.md) | verified | Documentation/QA engineer / codex/r28-inventory | Integrated in PR #302 (commit 5936aab); regenerated docs/IPC_REFERENCE.md to match 157 registered commands; CI passed; closed issue #292 |
| [R26](tasks/R26.md) | verified | QA engineer / codex/r26-ci-frontend-tests | Integrated in PR #301 (commit 2458021); added pnpm run test:stack to CI workflow; all matrix jobs passed; closed issue #108 |
| [R26-ipc-ci](tasks/R26-ipc-ci.md) | verified | QA engineer / codex/r26-ipc-ci | Integrated in PR #303 (commit 2f4c015); added IPC reference drift check to CI workflow; all matrix jobs passed; closed issue #286 |
| [R11](tasks/R11.md) | verified | Lead storage engineer / codex/r11-upsert-id | Integrated in PR #306 (commit 772efb1); replaced last_insert_rowid with RETURNING id in upsert_file; added unit tests for intervening inserts and tag/album/cleanup associations; CI passed; closed issue #241 |
| [R04-source](tasks/R04-source.md) | verified | Lead persistence/security engineer / codex/r04-source | Integrated in PR #307 (commit c4e0d72); implemented load_readonly in config_store.rs; source preserved byte-identically on success/failure/malformed; mock credential store used for headless Linux CI; multi-platform CI passed; closed issue #269 |
| [R01](tasks/R01.md) | verified | Lead recovery engineer / codex/r01-cloud-restore | Integrated in PR #312 (commit cd0f634); routed cloud restore through recovery::stage_restore; verified integrity, schema, and foreign keys; staged pending-restore database without clobbering live active WAL/mmap database; verified rollback creation and apply on restart; multi-platform CI passed; closed issue #98 |
| [R04](tasks/R04.md) | verified | Lead migration/configuration engineer / codex/r04-persistence | Integrated in PR #308 (commit 5d5f8a1); implemented save_migrated with destination-aware revision initialization; credential and persistence error propagation and readback validation; receipt artifact status distinguishing success/skipped/failed; preview_cleanup eligibility safeguard; multi-platform CI passed; closed issue #234 |
| [R05](tasks/R05.md) | verified | Lead persistence/migration engineer / codex/r05-empty-mask | Integrated in PR #309 (commit 2435c51); implemented is_database_empty; empty destination does not mask ambiguous/single source choice or retry; preview_migration and execute_migration_plan safely handle empty destination stubs; closed issue #235 |
| [R04-cleanup](tasks/R04-cleanup.md) | verified | Lead persistence/migration engineer / codex/r04-cleanup | Integrated in PR #310 (commit 23f384a); enforced independent destination config & model health validation for cleanup preview and execution; unit tests added; multi-platform CI passed; closed issue #270 |
| [R05-ui](tasks/R05-ui.md) | verified | Full-stack UI engineer / codex/r05-ui | Integrated in PR #311 (commit 65490e8); implemented LegacyMigrationModal.vue with source selection, lock status, pre-flight space check, progress indicator, error retry, and receipt; integrated access points in OnboardingModal, SettingsModal, MenuBar, and App; full 7-locale i18n support; multi-platform CI passed; closed issue #271 |
| [R33](tasks/R33.md) | verified | Lead file-service engineer / codex/r33-managed-import | Integrated in PR #313 (commit 62cb388); consolidated managed import publication service into crates/omera-scan::transform; enforced managed destination validation, collision policies, sidecar preservation, and interruption compensation; multi-platform CI passed; closed issue #262 |
| [R33-pipeline](tasks/R33-pipeline.md) | verified | Lead file-service engineer / codex/r33-pipeline | Integrated in PR #314 (commit 1613d9e); extracted pipeline harvesting & deferred cleanup business logic into crates/omera-scan::pipeline; enforced destination/source revalidation before trash; multi-platform CI passed; closed issue #293 |
| [R31](tasks/R31.md) | verified | Lead transform engineer / codex/r31-linked-destination | Integrated in PR #315 (commit 85da879); enforced managed vault folder validation in execute_managed_import_transform; rejecting linked, pipeline, and missing folders with zero side effects; multi-platform CI passed; closed issue #260 |
| [R08](tasks/R08.md) | verified | Lead transform/persistence engineer / codex/r08-disposition-order | Integrated in PR #316 (commit 5eaa2a3); enforced database update before original disposition in execute_library_batch_transform; indexed source kept intact on persistence failure with unindexed derivative compensation; archive collision renaming; multi-platform CI passed; closed issue #238 |
| [R32](tasks/R32.md) | verified | Rust/stack engineer / codex/r32-cull-trash-check | Integrated in PR #317 (commit e36201c); enforced filesystem trash success before database row deletion in cull_stack_drafts; honest completed counts returned and database records preserved on trash failure; multi-platform CI passed; closed issue #261 |
| [R02](tasks/R02.md) | verified | Lead pipeline engineer / codex/r02-pipeline-collision | Integrated in PR #318 (commit f3a60fd); enforced non-destructive SkipIdentical collision skipping and coordinated media/sidecar stem rename in harvest_pipeline_folder; protected pre-existing sidecars from overwrite; multi-platform CI passed; closed issue #232 |
| [R02-dedup](tasks/R02-dedup.md) | verified | Lead import/pipeline engineer / codex/r02-dedup-content-identity | Integrated in PR #319 (commit 9871ddd); implemented chunked byte content identity verification helper; resolved equal-size distinct file collisions without false reuse; multi-platform CI passed; closed issue #268 |
| [R03](tasks/R03.md) | unverified | Unassigned | Not executed by the split |
| [R06](tasks/R06.md) | unverified | Unassigned | Not executed by the split |
| [R07](tasks/R07.md) | unverified | Unassigned | Not executed by the split |
| [R36](tasks/R36.md) | unverified | Unassigned | Not executed by the split |
| [R07-export](tasks/R07-export.md) | unverified | Unassigned | Not executed by the split |
| [R07-sidecars](tasks/R07-sidecars.md) | unverified | Unassigned | Not executed by the split |
| [R09-privacy](tasks/R09-privacy.md) | unverified | Unassigned | Not executed by the split |
| [R09](tasks/R09.md) | unverified | Unassigned | Not executed by the split |
| [R10](tasks/R10.md) | unverified | Unassigned | Not executed by the split |
| [R23](tasks/R23.md) | unverified | Unassigned | Not executed by the split |
| [R10-discovery](tasks/R10-discovery.md) | unverified | Unassigned | Not executed by the split |
| [R12](tasks/R12.md) | unverified | Unassigned | Not executed by the split |
| [R16](tasks/R16.md) | unverified | Unassigned | Not executed by the split |
| [R34](tasks/R34.md) | unverified | Unassigned | Not executed by the split |
| [R13](tasks/R13.md) | unverified | Unassigned | Not executed by the split |
| [R14](tasks/R14.md) | unverified | Unassigned | Not executed by the split |
| [R25](tasks/R25.md) | unverified | Unassigned | Not executed by the split |
| [R25-estimate](tasks/R25-estimate.md) | unverified | Unassigned | Not executed by the split |
| [R25-export](tasks/R25-export.md) | unverified | Unassigned | Not executed by the split |
| [R14-webdav](tasks/R14-webdav.md) | unverified | Unassigned | Not executed by the split |
| [R14-s3](tasks/R14-s3.md) | unverified | Unassigned | Not executed by the split |
| [R24](tasks/R24.md) | unverified | Unassigned | Not executed by the split |
| [R24-cancel](tasks/R24-cancel.md) | unverified | Unassigned | Not executed by the split |
| [R17-mirror](tasks/R17-mirror.md) | unverified | Unassigned | Not executed by the split |
| [R17-revision](tasks/R17-revision.md) | unverified | Unassigned | Not executed by the split |
| [R17](tasks/R17.md) | unverified | Unassigned | Not executed by the split |
| [R18](tasks/R18.md) | unverified | Unassigned | Not executed by the split |
| [R18-concurrency](tasks/R18-concurrency.md) | unverified | Unassigned | Not executed by the split |
| [R19](tasks/R19.md) | unverified | Unassigned | Not executed by the split |
| [R21](tasks/R21.md) | unverified | Unassigned | Not executed by the split |
| [R30](tasks/R30.md) | unverified | Unassigned | Not executed by the split |
| [R35](tasks/R35.md) | unverified | Unassigned | Not executed by the split |
| [R29-table](tasks/R29-table.md) | unverified | Unassigned | Not executed by the split |
| [R37](tasks/R37.md) | unverified | Unassigned | Not executed by the split |
| [R20](tasks/R20.md) | unverified | Unassigned | Not executed by the split |
| [R22](tasks/R22.md) | unverified | Unassigned | Not executed by the split |
| [R22-sidebar](tasks/R22-sidebar.md) | unverified | Unassigned | Not executed by the split |
| [R22-labels](tasks/R22-labels.md) | unverified | Unassigned | Not executed by the split |
| [R29](tasks/R29.md) | unverified | Unassigned | Not executed by the split |
| [R20-denominator](tasks/R20-denominator.md) | unverified | Unassigned | Not executed by the split |
| [R29-tags](tasks/R29-tags.md) | unverified | Unassigned | Not executed by the split |
| [R15](tasks/R15.md) | unverified | Unassigned | Not executed by the split |
| [R28](tasks/R28.md) | verified | Documentation engineer / codex/r28-cloud-sync-claims | Integrated in PR #320 (commit 16f4fb4); reconciled cloud sync claims in README and wiki across 7 locales to upload-only mirroring, marked bidirectional sync planned, added regression tests; multi-platform CI passed; closed issue #257 |
| [R28-remote-db](tasks/R28-remote-db.md) | unverified | Unassigned | Not executed by the split |
| [R27](tasks/R27.md) | unverified | Unassigned | Not executed by the split |
