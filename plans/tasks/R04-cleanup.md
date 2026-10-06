# R04-cleanup / #270: Legacy cleanup eligibility lacks configuration and model destination validation

**Assignment:** one issue only. **Package:** [W03](../W03_PLAN.md).  
**Proposed priority:** P1 | **Owner:** Lead cleanup/security engineer | **Verification:** V3.  
**Tracking:** [#270](https://github.com/BerryUIKI/Omera/issues/270) | **Evidence at cutoff:** Source trace; latent route.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R04](R04.md), [R05](R05.md)

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Each cleanup candidate requires validated corresponding destination data and a separate explicit in-app user decision.

## Recommended implementation

1. Validate each destination artifact named in a migration receipt before marking its source eligible for cleanup; healthy SQLite does not validate configuration or model files.
2. Revalidate eligibility at execution time and exclude user media, external vaults and shared directories. Keep cleanup behind a separate explicit in-application decision.

## Targeted validation

- [ ] Inject missing, mismatched and incomplete destination config/model artifacts; preview and execution must reject cleanup.
- [ ] Test path allowlists, changed source identity and declined confirmation using synthetic application-data directories only.

## Issue acceptance (preserved verbatim)

- [ ] Validate configuration and model artifacts independently before marking them eligible.
- [ ] Reject missing, incomplete or mismatched destination artifacts.
- [ ] Keep user media, external vaults and shared directories outside automatic cleanup.

## Evidence and source pointers

**Code-confirmed latent cleanup defect.** Cleanup preview checks database health, then marks legacy configuration/model artifacts eligible without verifying their own migrated destination data. A healthy destination database does not establish that configuration or models were copied correctly. Current Vue code has no cleanup consumer; this is the exposed backend contract.

1. Use isolated legacy application-data roots and a healthy destination database.
2. Leave the migrated config absent or make a destination model copy incomplete.
3. Inspect cleanup eligibility without executing deletion.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Legacy cleanup eligibility lacks configuration and model destination validation. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src-tauri/src/legacy_migration.rs](D:/dev/Omera/src-tauri/src/legacy_migration.rs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
