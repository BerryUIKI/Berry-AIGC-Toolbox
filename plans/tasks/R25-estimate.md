# R25-estimate / #284: Export estimates retain the shared database mutex during decoding and encoding

**Assignment:** one issue only. **Package:** [W07](../W07_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Lead export/concurrency engineer | **Verification:** V3.  
**Tracking:** [#284](https://github.com/BerryUIKI/Omera/issues/284) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** None beyond baseline/ownership checks.

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Image estimation runs outside the shared database guard.

## Recommended implementation

1. Capture the required source/metadata DTO under a short lock and execute estimation decode/encode outside the shared database guard.
2. Fence stale estimates by source identity/generation and keep cancellation and decoding work bounded.

## Targeted validation

- [ ] Hold estimation at a controlled barrier and confirm unrelated queries complete.
- [ ] Change/delete the source during estimation and require a stale/canceled result instead of a misleading current estimate.

## Issue acceptance (preserved verbatim)

- [ ] Clone the required bounded input and release the guard before processing.
- [ ] Keep estimate progress/errors independent of database query access.
- [ ] Verify concurrent library reads remain responsive.

## Evidence and source pointers

**Code-confirmed.** estimate_export_file holds db(&state) while estimate_single_image performs image processing. A preview/estimate can block unrelated gallery operations that need the same mutex.

1. Trace the guard through estimate_single_image.
2. Use a slow synthetic image estimate and a concurrent library read.
3. Compare query latency while the estimate is running.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Export estimates retain the shared database mutex during decoding and encoding. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src-tauri/src/commands.rs](D:/dev/Omera/src-tauri/src/commands.rs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
