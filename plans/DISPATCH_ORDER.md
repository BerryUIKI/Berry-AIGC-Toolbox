# Combined dispatch order: Gemini and Opus 5.5

Updated October 7, 2026. Coordinator schedule based on the current implementation STATUS.md. This sets dispatch order only; it starts no agent, merges no branch and changes no completion state.

## Start now

1. Send the existing R12 startup prompt to Opus first. The current startup prompt assigns full R12 implementation and acceptance responsibility.
2. Immediately send the R28-remote-db startup prompt to Gemini. There is no need to wait for Opus to finish R12; documentation correction and scan implementation are independent.
3. R28 / #257 and R02-dedup / #268 are now integrated and verified. Do not redispatch them; the old #268 shared-module hold is cleared.

## Ordered queues

Each arrow is a separate one-issue assignment and fresh context, not one long batch prompt. Advance each model along its column. A row is an operating sequence, not an unrelated whole-package blocker: an independent next task may start as soon as its real prerequisites and file slot are ready. If both models would edit the same file, serialize those changes.

| Round | Gemini queue | Opus 5.5 queue | Dispatch/hand-off gate |
| --- | --- | --- | --- |
| 1 | [R28-remote-db](tasks/R28-remote-db.md) | [R12](tasks/R12.md) | Launch Opus first, then Gemini immediately. These tasks have no cross-model dependency. R12 is now a task-scoped lead implementation assignment; downstream work still waits for integrated acceptance. |
| 2 | [R22](tasks/R22.md) -> [R22-sidebar](tasks/R22-sidebar.md) -> [R29-tags](tasks/R29-tags.md) -> [R21](tasks/R21.md) | [R03](tasks/R03.md) -> [R06](tasks/R06.md) -> [R07](tasks/R07.md) -> [R07-export](tasks/R07-export.md) -> [R07-sidecars](tasks/R07-sidecars.md) -> [R09-privacy](tasks/R09-privacy.md) -> [R23](tasks/R23.md) -> [R09](tasks/R09.md) | UI and metadata lanes. The former #268 hold is cleared; check current shared-file ownership. Sidebar and gallery cards remain separate assignments. |
| 3 | [R34](tasks/R34.md) | [R10](tasks/R10.md) -> [R36](tasks/R36.md) | Gemini R34 waits for integrated R12 acceptance and the approved scan lifecycle. Check actual file overlap if classification needs new persistent provenance. |
| 4 | [R10-discovery](tasks/R10-discovery.md) | [R17-mirror](tasks/R17-mirror.md) -> [R17-revision](tasks/R17-revision.md) | Gemini AVIF discovery waits for R10 decoder/format acceptance. Opus establishes accepted config/revision behavior under the existing owner boundary. |
| 5 | [R17](tasks/R17.md) | [R18](tasks/R18.md) -> [R18-concurrency](tasks/R18-concurrency.md) | Gemini consumes integrated configuration fixes; Opus owns history.ts. Keep Settings and history contracts separate. |
| 6 | No new assignment / await handoff | [R19](tasks/R19.md) -> [R30](tasks/R30.md) -> [R35](tasks/R35.md) | Reserve App.vue/FileList.vue/VirtualGrid.vue for the Opus gallery lane. Gemini waits before taking its dependent gallery cards. |
| 7 | [R29-table](tasks/R29-table.md) -> [R22-labels](tasks/R22-labels.md) -> [R37](tasks/R37.md) | [R13](tasks/R13.md) -> [R14](tasks/R14.md) -> [R14-webdav](tasks/R14-webdav.md) -> [R14-s3](tasks/R14-s3.md) -> [R24](tasks/R24.md) -> [R24-cancel](tasks/R24-cancel.md) | Gemini takes gallery components after Opus hands off integrated query/stack state. Opus advances the sync lane; any overlapping config/IPC edit needs an explicit sequential owner. |
| 8 | No new assignment / await handoff | [R16](tasks/R16.md) | Watcher activation follows integrated cleanup/publication safety, with the existing lead-reviewed lifecycle contract. |
| 9 | No new assignment / await handoff | [R20](tasks/R20.md) -> [R20-denominator](tasks/R20-denominator.md) | Settle the statistics population and production DTOs before final localization. New SQL remains lead-reviewed. |
| 10 | [R29](tasks/R29.md) | [R25](tasks/R25.md) -> [R25-estimate](tasks/R25-estimate.md) -> [R25-export](tasks/R25-export.md) | Gemini localizes settled history/statistics/UI behavior. Opus works the lock-scope lane; shared files/locales remain single-owner. |
| 11 | No new assignment / await handoff | [R15](tasks/R15.md) | Backup scheduling waits for accepted config and safe recovery; production changes stay with the named lead owner. |
| 12 | No new assignment / await handoff | [R27](tasks/R27.md) | Signing/release qualification last in this operating queue. Earlier preparation is allowed, but qualification/sign-off requires the exact candidate and actual platform evidence. |

## Cross-model handoffs that must be complete

| Upstream | Then dispatch to Gemini |
| --- | --- |
| R12: integrated scan safety and approved service lifecycle | R34 |
| R10: verified shipped decoder/format policy | R10-discovery |
| R17-mirror + R17-revision: integrated authoritative config | R17 |
| R30: integrated stack/Table behavior | R29-table |
| R35: integrated selection scope/count contract | R22-labels |
| R21 + R36: masking policy and classification prerequisites | R37 |
| R17 + R18, plus settled statistics components for file coordination | R29 |

A lead-support analysis handoff does not itself satisfy an implementation prerequisite. The reserved owner must implement/integrate and verify that behavior before its dependent card proceeds. Other independent cards may continue while that gate is pending. Preserve the engineering ownership in [MODEL_ASSIGNMENT.md](MODEL_ASSIGNMENT.md) and each model queue.

## At every task boundary

Send only AGENT_RUNBOOK.md, the chosen task card and a concise prerequisite handoff. Require exact commit, changed production paths, actual acceptance results, failures/not-run cases and remaining gates. Agents must synchronize their own STATUS.md rows and affected coordination documentation without overwriting concurrent work. Validated task PR merge, safe task-branch cleanup and assigned-issue closure are now authorized under COMPLETION_WORKFLOW.md. Release publishing remains outside this assignment.

[Allocation overview](MODEL_ASSIGNMENT.md) | [Gemini roles](GEMINI_ASSIGNMENTS.md) | [Opus roles](OPUS_ASSIGNMENTS.md) | [Execution status](STATUS.md)

## Current startup prompts

[Gemini: R28-remote-db / #289](prompts/2026-10-07_GEMINI_R28_REMOTE_DB.md) and [Opus: R12 / #243](prompts/2026-10-07_OPUS_R12.md) can proceed in parallel. Follow [COMPLETION_WORKFLOW.md](COMPLETION_WORKFLOW.md) through documentation, integration, verification, branch cleanup and issue closure.
