# Gemini and Opus 5.5 task allocation

Updated October 7, 2026. This records task assignments and authorized completion steps; it does not itself execute application changes, acceptance checks or GitHub actions.

## Current evidence and scope

Use the updated implementation-repository [STATUS.md](STATUS.md). The Review directory's status is the original initialization and must not be used as current completion evidence. The local implementation tracker now records 23 verified topics and 44 unverified topics. R28 / #257 is merged in PR #320, and R02-dedup / #268 is merged in PR #319. The current startup issues #289 and #243 were checked and remain open.

Observed dev baseline: `9871dddb44cc1130d90737008c157c1ee8982a02`. Re-fetch current origin/dev at dispatch. The #268 implementation is integrated; the former branch hold is cleared. Start Gemini on R28-remote-db / #289 and Opus on R12 / #243.

| Queue | Topics | Delivery role |
| --- | --- | --- |
| [Gemini](GEMINI_ASSIGNMENTS.md) | 12 | Bounded UI, accessibility, localization, documentation and approved-service integration. |
| [Opus 5.5](OPUS_ASSIGNMENTS.md) | 32 | Complex state, concurrency, metadata, policy and cross-layer analysis; implementation only within the existing owner boundary. |
| Recorded verified | 23 | Excluded from fresh implementation assignments. |

Gemini's exact version was not supplied. This routing follows task coupling, data risk and the user's request to keep Gemini contexts small. It is not a measured head-to-head model result on Omera. General model capability references: [Anthropic Opus](https://www.anthropic.com/claude/opus), [Google Gemini model catalogue](https://ai.google.dev/gemini-api/docs/models).

## Ownership meaning

A model queue does not change the task's engineering owner. The repository [engineering handoff](../docs/ENGINEERING_HANDOFF.md) reserves persistence, cleanup, security and release implementations for the lead maintainer/Codex. Opus rows marked `Lead support` retain their existing scope except R12: the current human startup assignment appoints Opus as task-scoped lead implementation owner for R12 only. Other reserved production changes remain with their named lead owner. Where a specific lead-owned implementation assignment is already authorized, use that assignment's scope. No general UI assignment grants storage/cleanup/release authority.

Other Opus rows can implement their bounded behavior under the card owner and required lead contract review. Configuration semantics, new SQL/IPC, manual-override provenance and query-selection contracts must be checked before dependent UI changes. Preserve the original task dependencies and issue acceptance.

## First dispatches

Use [DISPATCH_ORDER.md](DISPATCH_ORDER.md) for the exact combined queues and cross-model handoff gates. Start Opus on R12, then immediately start Gemini on R28-remote-db; these two assignments are independent.

1. Gemini: R28-remote-db; R28 is already verified and must not be redispatched. Next: R22 menu keyboard behavior or R22-sidebar. R21 is also ready once FileList/VirtualGrid editing is reserved to Gemini.
2. Opus: R12 scan-completeness implementation and full acceptance first, under the current task-scoped lead assignment. Then R03 cleanup revalidation and R06/R07 metadata-policy work. Coordinate actual source ownership; #268 is now integrated and its former shared-module hold is cleared.
3. Gemini can take R34 after R12, R17 after the configuration lane, R10-discovery after R10, and dependent gallery/localization cards only after their listed prerequisites are integrated.
4. Opus keeps the history lane together: R18 -> R18-concurrency -> R19 -> R30 -> R35. Gemini then consumes the table/selection contract for R29-table and R22-labels. Sync comparison and transport use one sequential owner for cloud_sync.rs.

Avoid simultaneous edits to App.vue, SettingsModal.vue, VirtualGrid.vue, FileList.vue, Sidebar.vue, commands.rs, db.rs, transform.rs, pipeline.rs and cloud_sync.rs. A separate branch/context does not remove those conflicts. Prefer independent modules, or finish one owner lane before switching models.

## Assignment size and handoff

Both models receive one issue per assignment: [AGENT_RUNBOOK.md](AGENT_RUNBOOK.md), one task card and a short prerequisite handoff. Package plans and these queue indexes are coordinator material; do not paste all 13 or 32 cards into a model session. Start fresh for each issue, carrying forward only exact integrated commits/contracts and relevant remaining limits.

Return exact commit, before/after behavior, actual production-path/fault results, verification profile, not-run cases and remaining dependencies using [HANDOFF_TEMPLATE.md](HANDOFF_TEMPLATE.md). GUI/provider/platform qualification still requires its actual environment. Only integrated-candidate acceptance updates the completion tracker; this allocation does not modify it.

## Current completion authorization

The user now explicitly authorizes assigned agents to synchronize documentation, merge their own validated task PR into dev, safely delete their merged task branches, and close their assigned issue after integrated acceptance. Follow [COMPLETION_WORKFLOW.md](COMPLETION_WORKFLOW.md); earlier no-merge/no-close startup wording is superseded. Current prompts: [Gemini #289](prompts/2026-10-07_GEMINI_R28_REMOTE_DB.md), [Opus #243](prompts/2026-10-07_OPUS_R12.md).
