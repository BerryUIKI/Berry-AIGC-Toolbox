# Review reproductions — 2026-10-05

These are adversarial acceptance probes against the actual production Rust functions and TypeScript `ActionHistory` class. They intentionally fail on the reviewed `dev` baseline, `bdac8fb`. They are outside the main Cargo workspace and are not a replacement for the normal suite.

Relevant synthetic reproduction sources are published in their individual GitHub reports. The aggregate tracker #265 has been retired to keep one topic per issue. See ../GITHUB_PUBLICATION_2026-10-05.md for the current index.

Run from the repository root:

```powershell
cargo test --offline --manifest-path docs/reviews/repro-2026-10-05/Cargo.toml --target-dir target probes:: -- --test-threads=1
pnpm exec tsx --test docs/reviews/repro-2026-10-05/history-probes.test.mjs
pnpm exec tsx --test docs/reviews/repro-2026-10-05/history-concurrency.test.mjs
```

Remove `--offline` if the locked dependencies are not available locally. The harness has its own lockfile; it uses the repository's local crates and includes the actual `src-tauri/src/cloud_backup.rs` module. This lockfile is independent of the application's root lockfile. Dead-code warnings for unused cloud functions are expected in this smaller harness.

The adversarial unit probes use synthetic media, databases and backup archives in temporary directories. The archive test injects a database UPDATE failure using a temporary SQLite trigger. The cloud test keeps an active SQLite WAL connection open to reproduce the application's restore lifecycle; on this Windows machine it fails with OS error 1224 before replacement can complete. The later real_samples and real_operations binaries instead read the user-authorized AI image dataset and transform copied images in temporary directories; their outcomes and source-hash verification are recorded in ../REAL_SAMPLE_TESTS_2026-10-05.md.

Initial results on the review machine: 10 Rust probes fail and 2 TypeScript probes fail. Follow-up testing adds a failing negative-prompt classification Rust probe and a controlled failing TypeScript concurrency probe. These scenarios are not a count of independent user-facing issues. Two scenarios test different privacy policies. The managed-import helper probe tests a reusable service contract; that helper is not the current registered import IPC path.

Captured output: [Rust probe log](D:/dev/Omera/docs/reviews/repro-2026-10-05/rust-probe-results.txt), [TypeScript probe log](D:/dev/Omera/docs/reviews/repro-2026-10-05/history-probe-results.txt).

| Probe | Expected acceptance behavior | Observed baseline behavior |
| --- | --- | --- |
| `stripping_prompt_must_not_emit_prompt_text_sidecar` | StripPromptOnly suppresses prompt-bearing output | Confidential prompt emitted in `.txt` |
| `strip_ai_metadata_must_not_emit_prompt_text_sidecar` | StripAllAiMetadata suppresses prompt-bearing output | Confidential prompt emitted in `.txt` |
| `keep_supported_transform_must_preserve_embedded_prompt` | PNG-to-PNG resize retains supported prompt metadata | Embedded prompt becomes absent |
| `exported_avif_must_be_decodable_for_library_thumbnail` | App can thumbnail its own AVIF derivative | AVIF decoding unsupported |
| `managed_transform_import_must_reject_linked_destination` | Managed import rejects `link` destinations | Writes into linked directory and reports success |
| `archive_must_wait_until_database_update_succeeds` | Failed DB update preserves indexed source | Source archived; old indexed path missing |
| `transformed_database_dimensions_must_match_derivative` | Database dimensions match 32×16 derivative | Database retains original width 64 |
| `strip_all_batch_transform_must_not_retain_prompt_in_library` | StripAll leaves no prompt on transformed library record | Original private prompt remains in database |
| `upsert_existing_path_must_return_its_actual_id` | Updating A after inserting B returns A's ID | Returns B's ID |
| `cloud_restore_must_survive_active_wal_connection` | Restore succeeds safely with application's active lifecycle | Windows rejects live overwrite with error 1224 |
| Failed undo probe | Command remains available after rejection | Command disappears from undo stack |
| Failed redo probe | Command remains available after rejection | Command disappears from redo stack |
