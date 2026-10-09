# Remaining review fixes

Updated: 2026-10-10. Integration target: `dev`; `main` remains release-only.

This execution ledger supersedes the historical September dispatch status for the
October review reports. The handoff ownership boundaries, migration safeguards,
and release qualification requirements still apply. An implemented feature is
not evidence of native or platform qualification.

## Sequence and acceptance

Each row is a bounded PR (closely related failure cases may share one PR). Write
the behavior/ownership contract before implementation, commit coherent steps
with their tests, and preserve those commits through merge. Merge only after the
required CI checks pass; delete only the merged task branch.

| Order | Issues | Work and acceptance | Status |
| --- | --- | --- | --- |
| 0 | #267, #266, #247, #277, #278 | Reconcile fixes already integrated in #345–#348; verify source-preserving scan/ingestion and config failure/revision regressions | Integrated; issue reconciliation pending |
| 1 | #248 | Keep failed settings saves open, retain edits/revision, apply preferences only after success, reject overlapping submissions | PR #349; tests/build passed locally |
| 2 | #249, #279 | Preserve failed undo/redo in history and define serial execute/undo/redo ordering, including rejection and clear during a pending transition | Planned |
| 3 | #250 | Refresh active filters/counts after batch favorite/NSFW operations and undo/redo; retain newer selected metadata | Planned |
| 4 | #284 | Release the shared database guard before export estimation; run codecs on a blocking worker; concurrent reads must proceed | Planned |
| 5 | #285 | Bounded export input snapshot, codecs/output outside the shared guard; deterministic per-file results | Planned |
| 6 | #202 | Short transform read/write locks with codecs/publication outside them; retain transactional per-item safety | Planned |
| 7 | #255, #283 | Global transfer memory budget and cooperative cancellation across limiter/transfer waits; synthetic slow-transfer tests | Planned |
| 8 | #251, #280 | Populate model/sampler statistics from production queries and report genuinely analyzed files | Planned |
| 9 | #258 | Localize statistics, history and Sidebar text in all seven locales | Planned |
| 10 | #282 | Accurate per-image selection names for assistive technology | Planned |
| 11 | #295 | Explicit full-result selection or clearly named loaded-item selection; scope/navigation/cancellation tests | Planned |
| 12 | #259 | Table stack expansion with scoped members and keyboard access | Planned |
| 13 | #263 | Configurable Table columns at ordinary/narrow widths, compatible persistence defaults | Planned |
| 14 | #297 | Session masking toggle and guarded shortcut; reset temporary reveals on enabling masking; saved startup default stays explicit | Planned |
| 15 | #246 | Opt-in automatic backup scheduling, compatible defaults, snapshot safety, restart/failure status | Planned |
| 16 | #256 | Provision signed update pipeline without a trust bypass; validate tampering/missing signatures; record credentials/platform gates | Planned |

Safety and correctness precede convenience controls. Lock changes precede new
background backup scheduling. Shared gallery/App changes proceed sequentially.
Release signing cannot be declared complete without provisioned trust and actual
signed artifacts; record any unavailable external prerequisites explicitly.

## Verification evidence

- Settings follow-up on 2026-10-10: production `saveSettings` handler tests cover
  failed saves and duplicate submissions; `pnpm run build` and `pnpm run
  test:stack` pass (218 tests). Vite reports its existing large bundle warning.
- Cross-layer changes require build, frontend tests, formatting, workspace clippy
  and workspace Rust tests. Regenerate IPC inventory after command changes and
  check DTO compatibility separately.
- Gallery changes require Grid/Waterfall/Table checks at narrow/wide sizes,
  rapid dragging, stack transitions, keyboard navigation and reduced motion.
  Record actual browser/native evidence separately from helper tests.
- Preserve unrelated local files, external media, vaults and all legacy source
  data. Optional legacy cleanup still requires its separate in-application
  decision and validated destination.
