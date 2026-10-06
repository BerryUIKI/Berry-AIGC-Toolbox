# R25-export / #285: Batch export retains the shared database mutex throughout transcoding and output writes

**Assignment:** one issue only. **Package:** [W07](../W07_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Lead export/concurrency engineer | **Verification:** V3.  
**Tracking:** [#285](https://github.com/BerryUIKI/Omera/issues/285) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R06](R06.md), [R07-export](R07-export.md)

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Export processing and output writes do not retain the shared application database guard.

## Recommended implementation

1. Capture bounded/paged export inputs without holding AppState.db across decoding, parallel transcoding or destination writes.
2. Validate source identity as needed and report item-level failures/cancellation without reacquiring one long lock for the entire batch.

## Targeted validation

- [ ] Pause a production export stage and verify gallery/config queries remain usable.
- [ ] Test large query batches, concurrent changes, policy outputs and partial failures using copies; ensure input capture is not an unbounded library query.

## Issue acceptance (preserved verbatim)

- [ ] Load bounded export inputs before starting codec/output work.
- [ ] Use an explicit connection/data snapshot contract for export.
- [ ] Measure concurrent query latency during export.

## Evidence and source pointers

**Code-confirmed.** export_files_batch obtains db_guard and passes it into execute_batch_export, retaining it until export completes. Parallel raster processing and destination writes therefore serialize unrelated AppState.db users behind the entire job.

1. Trace db_guard through execute_batch_export.
2. Run a slow synthetic multi-image export into a temporary destination.
3. Issue concurrent gallery/search reads using the shared database.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Batch export retains the shared database mutex throughout transcoding and output writes. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src-tauri/src/commands.rs](D:/dev/Omera/src-tauri/src/commands.rs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
