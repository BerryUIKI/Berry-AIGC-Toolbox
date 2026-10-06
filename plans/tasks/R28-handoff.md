# R28-handoff / #290: Engineering handoff reports an outdated identity migration status

**Assignment:** one issue only. **Package:** [W01](../W01_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Documentation engineer with lead sign-off | **Verification:** V5.  
**Tracking:** [#290](https://github.com/BerryUIKI/Omera/issues/290) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** None beyond baseline/ownership checks.

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

The handoff describes current completed and remaining migration work.

## Recommended implementation

1. Reconcile the handoff's identity/package implementation status with the current source and verified acceptance evidence.
2. Preserve lead/general-engineer ownership and pre-1.0 compatibility requirements. Distinguish implemented source, incomplete acceptance and future work.

## Targeted validation

- [ ] Cross-check runtime identifiers, crate names and migration registrations against the exact candidate commit.
- [ ] Ensure unfinished installer, recovery and security acceptance is not mislabeled complete.

## Issue acceptance (preserved verbatim)

- [ ] Update the baseline and identity completion status.
- [ ] Keep legacy discovery/import obligations explicit.
- [ ] Retain current ownership boundaries and actionable remaining acceptance criteria.

## Evidence and source pointers

**Code-confirmed documentation mismatch.** ENGINEERING_HANDOFF reports that runtime/crate identity migration has not completed, despite current Omera identifiers, com.berryuiki.omera and omera-* crates. This handoff is the ownership/acceptance entry point and can send engineers to already completed work.

1. Compare the handoff status/baseline with runtime identifiers and workspace crate names.
2. Identify completed identity changes separately from remaining discovery/import compatibility work.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Engineering handoff reports an outdated identity migration status. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md), [src-tauri/tauri.conf.json](D:/dev/Omera/src-tauri/tauri.conf.json), [Cargo.toml](D:/dev/Omera/Cargo.toml). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
