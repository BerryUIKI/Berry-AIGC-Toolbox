# R05-ui / #271: Provide a legacy source-selection and retry workflow in the application

**Assignment:** one issue only. **Package:** [W03](../W03_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Migration UI engineer; lead owns backend contracts | **Verification:** V3.  
**Tracking:** [#271](https://github.com/BerryUIKI/Omera/issues/271) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R04](R04.md), [R05](R05.md), [R28-api](R28-api.md)

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Users can inspect migration state, choose a supported source and retry without manually moving or deleting application data.

## Recommended implementation

1. Consume the implemented migration status/preview/start contracts for candidate source selection, progress, failure and retry.
2. Keep cleanup separate from successful import and show preserved source state. Do not invent proposed IPC commands or combine unrelated storage-backend choices with legacy source selection.

## Targeted validation

- [ ] Mount the actual workflow for zero/one/multiple candidates, inaccessible sources, partial failure, restart and retry.
- [ ] Validate native keyboard/focus behavior and exact DTO compatibility with the lead-owned services.

## Issue acceptance (preserved verbatim)

- [ ] Expose status and preview using implemented IPC DTOs.
- [ ] Provide source selection and actionable retry/error states.
- [ ] Preserve the source and require a separate explicit decision for cleanup.

## Evidence and source pointers

**Code-confirmed feature gap.** The backend registers legacy migration status, preview and start commands, but Vue has no consumers for them. Users with multiple discovered legacy libraries cannot choose a source or retry a failed migration through the intended application workflow.

1. Use isolated application-data roots with multiple supported legacy candidates.
2. Open the app and inspect onboarding/settings for source selection and retry.
3. Exercise preview and cancellation before starting any import.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Provide a legacy source-selection and retry workflow in the application. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src-tauri/src/lib.rs](D:/dev/Omera/src-tauri/src/lib.rs), [src/App.vue](D:/dev/Omera/src/App.vue), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
