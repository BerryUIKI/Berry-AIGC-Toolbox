# Opus 5.5 assignments

Updated October 7, 2026. Thirty-two remaining topics are assigned to this queue. R02-dedup is excluded because it is now integrated and verified in PR #319. Read the [allocation overview](MODEL_ASSIGNMENT.md) before dispatch.

`Lead support` means reproduce, design, add safe fault fixtures and review within the current human assignment; the lead maintainer/Codex retains reserved production implementation and release/cleanup authority. `Implement / contract review` means the named card owner may implement its bounded behavior after relevant lead-owned contracts are settled. Neither mode transfers ownership or removes issue acceptance.

## Publication and privacy

| Task | Role | Prerequisite gate |
| --- | --- | --- |
| [R03](tasks/R03.md) / #233 - Deferred pipeline cleanup does not revalidate the destination or source identity | Lead support | Ready after candidate/owner checks |
| [R06](tasks/R06.md) / #236 - Export privacy modes leak the prompt into text sidecars | Lead support | Ready after candidate/owner checks |
| [R07](tasks/R07.md) / #237 - KeepSupported transformation drops embedded generation metadata | Implement / contract review | Ready after candidate/owner checks |
| [R07-export](tasks/R07-export.md) / #272 - KeepAll export strips embedded metadata during re-encoding | Implement / contract review | [R07](tasks/R07.md) |
| [R07-sidecars](tasks/R07-sidecars.md) / #273 - Managed import copies prompt-bearing sidecars despite stripping policies | Lead support | [R06](tasks/R06.md), [R07](tasks/R07.md) |
| [R09-privacy](tasks/R09-privacy.md) / #274 - StripAll batch transformation retains the original private prompt in SQLite | Lead support | [R07](tasks/R07.md) |
| [R23](tasks/R23.md) / #254 - Managed import bypasses processing for advanced or metadata-only transform settings | Lead support | [R07-sidecars](tasks/R07-sidecars.md) |
| [R09](tasks/R09.md) / #239 - Batch transformation leaves database dimensions at their original values | Lead support | [R07](tasks/R07.md) |

## Formats and classification

| Task | Role | Prerequisite gate |
| --- | --- | --- |
| [R10](tasks/R10.md) / #240 - The app can create AVIF files that its thumbnail decoder cannot read | Implement / contract review | [R07](tasks/R07.md) |
| [R36](tasks/R36.md) / #296 - Negative prompt exclusions trigger false-positive NSFW classification | Implement / contract review | Ready after candidate/owner checks |

## Scan and watcher lifecycle

| Task | Role | Prerequisite gate |
| --- | --- | --- |
| [R12](tasks/R12.md) / #243 - An incomplete filesystem walk is treated as proof that indexed files disappeared | Task-scoped lead implementation | Ready after candidate/owner checks |
| [R16](tasks/R16.md) / #247 - Real-time pipeline ingestion is configured but never triggered | Implement / contract review | [R03](tasks/R03.md) |

## Sync namespace, verification and transport

| Task | Role | Prerequisite gate |
| --- | --- | --- |
| [R13](tasks/R13.md) / #244 - Cloud object keys collide between library roots with the same basename | Lead support | Ready after candidate/owner checks |
| [R14](tasks/R14.md) / #245 - FastFingerprint sync skips same-length content changes | Implement / contract review | Ready after candidate/owner checks |
| [R14-webdav](tasks/R14-webdav.md) / #275 - WebDAV delta sync ignores the selected checksum strategy | Implement / contract review | [R14](tasks/R14.md) |
| [R14-s3](tasks/R14-s3.md) / #276 - S3 checksum mode treats a missing remote digest as a verified size match | Implement / contract review | [R14](tasks/R14.md) |
| [R24](tasks/R24.md) / #255 - Cloud sync allocates whole files without a global memory budget | Lead support | [R14](tasks/R14.md) |
| [R24-cancel](tasks/R24-cancel.md) / #283 - Cloud sync cancellation does not interrupt limiter waits or active transfers | Lead support | [R24](tasks/R24.md) |

## Database lock scope

| Task | Role | Prerequisite gate |
| --- | --- | --- |
| [R25](tasks/R25.md) / #202 - Batch transformations retain the shared database mutex during image processing | Lead support | Ready after candidate/owner checks |
| [R25-estimate](tasks/R25-estimate.md) / #284 - Export estimates retain the shared database mutex during decoding and encoding | Lead support | Ready after candidate/owner checks |
| [R25-export](tasks/R25-export.md) / #285 - Batch export retains the shared database mutex throughout transcoding and output writes | Lead support | [R06](tasks/R06.md), [R07-export](tasks/R07-export.md) |

## Authoritative configuration

| Task | Role | Prerequisite gate |
| --- | --- | --- |
| [R17-mirror](tasks/R17-mirror.md) / #277 - Failed configuration saves overwrite the localStorage mirror | Lead support | Ready after candidate/owner checks |
| [R17-revision](tasks/R17-revision.md) / #278 - Settings save reloads the latest revision before overwriting stale form values | Lead support | Ready after candidate/owner checks |

## History and gallery query state

| Task | Role | Prerequisite gate |
| --- | --- | --- |
| [R18](tasks/R18.md) / #249 - Failed undo or redo removes the command from history | Implement / contract review | Ready after candidate/owner checks |
| [R18-concurrency](tasks/R18-concurrency.md) / #279 - ActionHistory permits overlapping undo transitions | Implement / contract review | [R18](tasks/R18.md) |
| [R19](tasks/R19.md) / #250 - Batch favorite/NSFW changes and their undo leave filters and counters stale | Implement / contract review | [R18](tasks/R18.md), [R18-concurrency](tasks/R18-concurrency.md) |
| [R30](tasks/R30.md) / #259 - Table mode hides collapsed stack members without an expansion affordance | Implement / contract review | [R19](tasks/R19.md) |
| [R35](tasks/R35.md) / #295 - Select All silently selects only the loaded page in a larger gallery | Implement / contract review | [R19](tasks/R19.md), [R30](tasks/R30.md) |

## Statistical populations

| Task | Role | Prerequisite gate |
| --- | --- | --- |
| [R20](tasks/R20.md) / #251 - Model and sampler statistics tabs receive hardcoded empty arrays | Implement / contract review | Ready after candidate/owner checks |
| [R20-denominator](tasks/R20-denominator.md) / #280 - Prompt statistics counts all indexed files as analyzed | Implement / contract review | [R20](tasks/R20.md) |

## Backup and release

| Task | Role | Prerequisite gate |
| --- | --- | --- |
| [R15](tasks/R15.md) / #246 - Implement the configured automatic cloud backup scheduler | Lead support | [R17](tasks/R17.md) |
| [R27](tasks/R27.md) / #256 - The checked-in release pipeline does not provision signed automatic updates | Lead support | Ready after candidate/owner checks |

## Sequence and contract limits

- Prioritize ready P1 safety work. Start with the complete R12 scan failure/reconciliation fix under the current task-scoped lead assignment. #268 is integrated; its former pipeline/transform hold is cleared. Privacy preservation is a bytes/sidecars/SQLite contract, not just a field rename.
- Keep R06/R07 -> R07-export/R07-sidecars -> R09-privacy/R23/R09 as focused cards. Do not combine all metadata changes into one rewrite. R10 needs real shipped decoder/target evidence before Gemini R10-discovery.
- Keep R14 -> R14-webdav/R14-s3 -> R24 -> R24-cancel under one comparison/transport contract and serialize cloud_sync.rs edits. R13 remote namespaces require a source-preserving compatibility design.
- Keep R17-mirror and R17-revision under one authoritative config contract before Gemini R17. Keep R18 -> R18-concurrency -> R19 -> R30 -> R35 coherent; send Gemini a concise accepted state/selection handoff.
- For R35, the existing acceptance permits an explicitly labeled loaded-only action; do not invent a new persistent/full-query selection service when that smaller option is the chosen product contract. A full-query backend needs the lead-owned contract first.
- R36 can repair structured classification; new manual-override persistence and bulk reclassification remain gated. R20/R20-denominator need a consistent analyzed population; new bounded SQL/DTO designs require lead review.
- R15 waits for accepted configuration and safe snapshot/restore lifecycle. R27 can prepare signing/recovery qualification; keys, publishing and release sign-off remain with the release owner. Neither task is satisfied by mocked UI success.

## Copyable prompt

```text
Repository: D:\dev\Omera. Assigned model: Opus 5.5. Task: <ONE ID>.
Read plans/AGENT_RUNBOOK.md and plans/tasks/<ID>.md.
Confirm the task owner, exact dev baseline, prerequisites and shared-file slot.
State the invariant and current production failure before proposing a change.
For Lead support: deliver reproduction/fault fixtures and a concrete design/review
for the reserved owner; do not execute reserved production writes without that assignment.
For implementation or task-scoped lead implementation: make the smallest change satisfying this card, preserve DTO/data
compatibility, and run its targeted fault/concurrency checks plus required profile.
Return HANDOFF_TEMPLATE.md fields, integrated evidence and all not-run limitations.
Do not execute adjacent cards or claim an untested platform/provider passed.
```

[Allocation overview](MODEL_ASSIGNMENT.md) | [Gemini queue](GEMINI_ASSIGNMENTS.md) | [Execution status](STATUS.md)

R12 now has a full implementation assignment, superseding its earlier support-only role. Follow [COMPLETION_WORKFLOW.md](COMPLETION_WORKFLOW.md) and the [current Opus R12 startup prompt](prompts/2026-10-07_OPUS_R12.md). Other support-only roles are unchanged.
