# Remaining review fixes

Updated: 2026-10-11. Integration target: `dev`; `main` remains release-only.

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
| 0 | #267, #266, #247, #277, #278 | Reconcile fixes already integrated in #345–#348; verify source-preserving scan/ingestion and config failure/revision regressions | Integrated; five issues reconciled |
| 1 | #248 | Keep failed settings saves open, retain edits/revision, apply preferences only after success, reject overlapping submissions | Merged #349; CI passed on three platforms |
| 2 | #249, #279 | Preserve failed undo/redo in history and define serial execute/undo/redo ordering, including rejection and clear during a pending transition | Merged #351; CI passed on three platforms |
| 3 | #250 | Refresh active filters/counts after batch favorite/NSFW operations and undo/redo; retain newer selected metadata | Merged #352; CI and synthetic browser checks passed |
| 4 | #284 | Release the shared database guard before export estimation; run codecs on a blocking worker; concurrent reads must proceed | Merged #353; CI passed on three platforms |
| 5 | #285 | Bounded export input snapshot, codecs/output outside the shared guard; deterministic per-file results | Merged #354; CI passed on three platforms |
| 6 | #202 | Short transform read/write locks with codecs/publication outside them; retain transactional per-item safety | Merged #355; CI passed on three platforms |
| 7 | #255, #283 | Global transfer memory budget and cooperative cancellation across limiter/transfer waits; synthetic slow-transfer tests | Merged #357 and #358; CI passed on three platforms |
| 8 | #251, #280 | Populate model/sampler statistics from production queries and report genuinely analyzed files | Merged #356; CI passed on three platforms |
| 9 | #258 | Localize statistics, history and Sidebar text in all seven locales | Merged #361; CI passed on three platforms |
| 10 | #282 | Accurate per-image selection names for assistive technology | Merged #359; CI and synthetic browser checks passed |
| 11 | #295 | Explicit full-result selection or clearly named loaded-item selection; scope/navigation/cancellation tests | Merged #360; CI and synthetic browser checks passed |
| 12 | #259 | Table stack expansion with scoped members and keyboard access | Merged #362; CI and synthetic browser checks passed |
| 13 | #263 | Configurable Table columns at ordinary/narrow widths, compatible persistence defaults | Merged #363; CI and synthetic browser checks passed |
| 14 | #297 | Session masking toggle and guarded shortcut; reset temporary reveals on enabling masking; saved startup default stays explicit | Merged #364; CI and synthetic browser checks passed |
| 15 | #246 | Opt-in automatic backup scheduling, compatible defaults, snapshot safety, restart/failure status | Snapshot foundation checked; scheduler next |
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

## Individual selection accessibility contract

Grid/Waterfall and Table selection controls name the individual file and current
operation (select/deselect), using all seven locales. A per-file control never
uses the bulk selection label. Checkbox checked state agrees with membership in
the shared selection set. Header/batch selection labels remain separate and their
loaded-result scope is handled in #295. Rendering tests use production components
with both selected and unselected file fixtures.

- Loaded selection on 2026-10-11: synthetic 1,200-item browser library at
  960 × 640 and 1600 × 950; Grid/Waterfall/Table retain 398 selected while
  paging raises the loaded count to 798, then explicit repeat selects 798.
  Favorites reports 10 selectable loaded representations of 12 results with
  a collapsed three-member stack. Browser checks do not qualify native IPC.


## Review feedback localization contract

Statistics loading/empty states, actions, hints and accessible names use the
selected locale, alongside Sidebar navigation/tools and panel toggles. History
records retain the translated action name created when the operation runs;
undo/redo notifications use the current locale around that recorded name.
All seven supported locales supply the same keys. Backend errors and user data
remain verbatim. Production component rendering and operation-handler tests
exercise localized text rather than English-only fixture fallbacks.

- Table stacks on 2026-10-11: synthetic browser checks at 960 × 640 and
  1600 × 950 confirm Enter expansion, Space collapse, 25 mounted rows at the
  narrow size, and consistent expanded members across Grid/Waterfall/Table.
  Existing Table horizontal overflow is handled separately in #263.

- Table columns on 2026-10-11: build, 247 frontend tests, formatting, all-targets
  workspace clippy and workspace Rust tests pass. Serde/TypeScript preference
  fixtures verify old defaults and round-trip field names; IPC inventory remains
  159 commands. Synthetic browser checks at 960 × 640, 1280 × 850 and 1600 × 950
  confirm compact columns fit with both side panes, 46-pixel rows for long names,
  custom visibility/width saves, keyboard row navigation and internal horizontal
  scrolling (558 pixels) for deliberately wider preferences.

- Session masking on 2026-10-11: synthetic browser checks at 960 × 640 and
  1600 × 950 confirm the control remains visible, state survives Favorites and
  Grid/Waterfall/Table changes, enabling conceals prior reveals, and Shift+M
  is ignored in search input and a statistics dialog. The toolbar wraps controls
  to retain the explicit state at narrow widths. Native IPC remains separate.

- Snapshot foundation on 2026-10-11: build, all 249 frontend tests (after one
  isolated process exit passed on rerun), formatting, all-targets workspace clippy
  and workspace Rust tests pass. WAL data/counts, unique archive names, ordinary
  failure cleanup, preserved destination/source data, and streamed S3/WebDAV
  provider paths/content lengths are covered. Automatic scheduling is not yet
  implemented by this foundation.
