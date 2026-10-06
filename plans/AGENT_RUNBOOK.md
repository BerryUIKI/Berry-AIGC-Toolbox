# Agent runbook

Use this file with exactly one assigned task card. The overall review, package plans and reports are coordinator/reference material; reading them does not assign their tasks. The current human assignment and repository AGENTS.md determine authorized work.

## Start and scope

1. Work in `D:\dev\Omera`. Record branch, HEAD and `git status --short`; preserve all unrelated changes. Feature/fix branches start from and target `dev`, normally using `codex/`. `main` is release-only.
2. Read repository `AGENTS.md`, relevant ownership/acceptance in `docs/ENGINEERING_HANDOFF.md`, and relevant order in `docs/DELIVERY_ROADMAP.md`. Read only the additional migration/storage/performance sections applicable to this task. Stale status prose is evidence to verify, not permission to discard safety rules.
3. Confirm the assigned owner, current production path and integrated prerequisite evidence. A referenced card is a dependency, not another assignment. Reproduce safely on the actual candidate; review-cutoff evidence does not prove the current defect still exists.
4. Implement one issue's behavior and its necessary tests/docs. The listed implementation steps are recommended approaches; the issue acceptance remains the closure contract. If already fixed, verify and report that disposition instead of duplicating code.
5. For a missing prerequisite/contract, describe the smallest dependency and continue independent reproduction/fixtures. Do not invent an IPC command, expand into neighboring fixes or silently remove dependency edges. A missing lead-owned design should be made concrete for review.

## Boundaries

- Persistence, identity, cleanup, security, release and other reserved high-impact work remain lead-owned. A generic UI assignment does not authorize those implementations.
- Keep reusable business rules out of Tauri adapters/templates. Release shared database guards before expensive image/filesystem/network work; preserve per-item consistency.
- Read `docs/API_CONTRACTS.md` and `docs/IPC_REFERENCE.md` before IPC edits. Regenerate inventory after command changes and validate DTO compatibility separately. Advisory contracts in `CONTRACTS.md` are not implemented commands.
- Keep SQLite migrations append-only, legacy import/discovery readable before 1.0, and config defaults compatible in frontend/backend. Preserve source data. Cleanup requires validated destination proof and a separate explicit decision in the application; exclude user media, external vaults and shared directories from automatic legacy application-data cleanup.
- Use synthetic temporary fixtures or authorized copies for fault injection. Keep live user data/cloud namespaces outside destructive tests.
- Preserve visible-window gallery work, fixed card widths, bounded/deduplicated decoding, SQLite-backed first paint and reduced motion. UI text belongs in locale resources; engineering docs are English. Update lifecycle/caching/persistence docs when behavior changes.
- Coordinate one active editor for shared files, especially `App.vue`, `VirtualGrid.vue`, `FileList.vue`, `SettingsModal.vue`, `commands.rs`, `cloud_sync.rs`, `db.rs` and `transform.rs`. Separate contexts/branches do not remove integration conflicts.

## Verification

Run the assigned profile from `D:\dev\Omera`; upgrade it if the actual diff crosses layers. Record failures, missing environments and not-run cases. Use production services/helpers/components, rather than copied behavior or source strings as proof.

| Profile | Required evidence |
| --- | --- |
| V0 | Behavior-free formatting diff; `cargo fmt --check`; affected CI gets past formatting. |
| V1 | Targeted Rust/service/fault cases; fmt, workspace Clippy with warnings denied, workspace Rust tests. Upgrade to V3 for IPC/config/UI/lifecycle coupling. |
| V2 | `pnpm run build`, `pnpm run test:stack`, relevant maintained component/helper regressions; native keyboard/focus cases when affected. |
| V3 | All five commands below plus targeted integration/fault cases; separate DTO compatibility for changed IPC. |
| V4 | V3 plus native Grid/Waterfall/Table at narrow/wide sizes, rapid scrollbar dragging, stack expansion/collapse, keyboard navigation and reduced motion; bounded query/selection scope. |
| V5 | Verify docs/CI/contracts against actual candidate; relevant generator/drift/schema checks; documentation checks cannot prove runtime acceptance. |
| V6 | V3 plus supported-target installer/update/recovery, required CI variants and valid/tampered/wrong-key signatures; lead release qualification. |

```powershell
pnpm run build
pnpm run test:stack
cargo fmt --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

IPC changes also require `node scripts/generate-ipc-reference.mjs --check`. Run maintained memory/scroll suites and CI variants when required by the relevant repository handoff. Helper timings do not establish native frame performance.

## Handoff

Use `HANDOFF_TEMPLATE.md`: exact commit, before/after behavior, changed production paths, targeted outcomes, command results, compatibility/source preservation, remaining dependencies and not-run limits. Each acceptance belongs to its issue; a safe alternate route or ordinary build is insufficient for a latent helper/command finding.

Update `STATUS.md` only with actual execution evidence. An Agent handoff may be implementation-ready; verified issue completion requires integration and acceptance on the integrated candidate. Publishing, messaging GitHub and issue closure require authorization in the executing assignment. Planning alone authorizes none of those actions.
