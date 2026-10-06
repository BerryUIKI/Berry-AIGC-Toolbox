# Gemini assignments

Updated October 7, 2026. Twelve remaining topics are assigned to this queue. Current states are dispatch gates derived from the implementation STATUS.md, not a claim that each defect was reproduced today. Read the [allocation overview](MODEL_ASSIGNMENT.md) for scope and shared-file ownership.

One session = runbook + one card + concise prerequisite evidence. Keep implementation limited to the named behavior; refer unresolved storage/IPC/lifecycle contracts to their existing lead owner.

| Task | Bounded deliverable | Prerequisite gate | Scope constraint |
| --- | --- | --- | --- |
| [R28-remote-db](tasks/R28-remote-db.md) / #289 | Correct SQLite versus remote-backend claims | Ready after candidate/owner checks | Documentation only; do not implement a remote database. |
| [R22](tasks/R22.md) / #253 | Menu keyboard navigation and dismissal | Ready after candidate/owner checks | MenuBar component; focus return and listener teardown. |
| [R22-sidebar](tasks/R22-sidebar.md) / #281 | Sidebar keyboard access | Ready after candidate/owner checks | Sidebar controls; keep navigation and rendering bounded. |
| [R21](tasks/R21.md) / #252 | Table sensitive-content masking | Ready after candidate/owner checks | Share existing presentation state; do not change classification. |
| [R29-tags](tasks/R29-tags.md) / #264 | Sidebar tag search | [R22-sidebar](tasks/R22-sidebar.md) | After keyboard access; preserve selected tags and bounded queries. |
| [R17](tasks/R17.md) / #248 | Settings save error and retry feedback | [R17-mirror](tasks/R17-mirror.md), [R17-revision](tasks/R17-revision.md) | Consume accepted config/revision service; no independent persistence policy. |
| [R34](tasks/R34.md) / #266 | Populated-folder initial scan UI integration | [R12](tasks/R12.md) | Use the approved scan service after R12; no ad-hoc traversal or reconciliation. |
| [R29-table](tasks/R29-table.md) / #263 | Table column visibility and sizing | [R30](tasks/R30.md) | After R30; use compatible frontend preferences; new backend config needs lead contract. |
| [R37](tasks/R37.md) / #297 | Toolbar masking toggle | [R21](tasks/R21.md), [R36](tasks/R36.md) | After R21/R36; session presentation only; never reclassify files. |
| [R22-labels](tasks/R22-labels.md) / #282 | Item-specific accessible selection labels | [R35](tasks/R35.md) | After R35; names/state in all gallery modes. |
| [R29](tasks/R29.md) / #258 | Localize statistics/history/Sidebar text | [R17](tasks/R17.md), [R18](tasks/R18.md) | After config/history behavior settles; all supported locales. |
| [R10-discovery](tasks/R10-discovery.md) / #267 | AVIF scanner discovery coverage | [R10](tasks/R10.md) | After approved/shipped decoder policy; extension/container fixtures only, no pruning changes. |

## Suggested queue

Start R28-remote-db, then R22 or R22-sidebar. R28 is already merged and verified; preserve its cloud-sync correction. After R22-sidebar, R29-tags can follow in the same component ownership lane. R21 is ready after reserving the gallery components; coordinate it with Opus gallery work. The other rows wait for the listed integrated prerequisites, not merely an unmerged implementation.

For R34, reuse the approved scan command/event lifecycle. If the backend cannot supply its required contract, report that gap instead of building reconciliation in Vue. For R29-table, propose compatible defaults before adding any backend config field. For R10-discovery, keep deletion/reconciliation semantics unchanged and require actual decoder support from R10.

## Copyable prompt

```text
Repository: D:\dev\Omera. Assigned model: Gemini. Task: <ONE ID>.
Read plans/AGENT_RUNBOOK.md and plans/tasks/<ID>.md.
Confirm current dev baseline, integrated prerequisites and file ownership.
Implement only this card and its necessary tests/docs.
Use existing approved services; report a missing contract with a minimal reproduction.
Do not expand into neighboring tasks or change lead-owned persistence/cleanup/release.
Return HANDOFF_TEMPLATE.md fields with actual results and not-run cases.
```

[Allocation overview](MODEL_ASSIGNMENT.md) | [Opus queue](OPUS_ASSIGNMENTS.md) | [Execution status](STATUS.md)

Follow the authorized merge/documentation/cleanup/closure workflow in [COMPLETION_WORKFLOW.md](COMPLETION_WORKFLOW.md). Current startup: [Gemini R28-remote-db](prompts/2026-10-07_GEMINI_R28_REMOTE_DB.md).
