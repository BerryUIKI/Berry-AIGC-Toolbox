# R26-ipc-ci / #286: CI does not check the generated IPC inventory for drift

**Assignment:** one issue only. **Package:** [W01](../W01_PLAN.md).  
**Proposed priority:** P2 | **Owner:** QA engineer | **Verification:** V5.  
**Tracking:** [#286](https://github.com/BerryUIKI/Omera/issues/286) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R28-inventory](R28-inventory.md)

**Downstream consumers:** [R27](R27.md) Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

CI detects stale generated command inventory.

## Recommended implementation

1. Add the existing IPC inventory generator's check mode to CI so registered-command drift fails the appropriate job.
2. Keep DTO compatibility as an additional independent contract check; generated command names alone cannot validate payloads.

## Targeted validation

- [ ] Change a registered command in a temporary branch without regeneration and confirm CI fails.
- [ ] Regenerate the inventory, confirm check mode passes, and separately execute relevant DTO contract tests.

## Issue acceptance (preserved verbatim)

- [ ] Run the inventory --check entry point in CI.
- [ ] Fail on generated inventory drift.
- [ ] Keep DTO compatibility validation separate from command inventory generation.

## Evidence and source pointers

**Code-confirmed CI gap.** The repository provides an IPC inventory generator/check mode, but CI never runs its drift check. Command-surface changes can leave IPC documentation stale without a failing check.

1. Inspect the inventory check script and CI workflow.
2. Change a registered command in an isolated validation branch without regenerating the inventory.
3. Run the local check and compare with CI coverage.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: CI does not check the generated IPC inventory for drift. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [.github/workflows/ci.yml](D:/dev/Omera/.github/workflows/ci.yml), [scripts/generate-ipc-reference.mjs](D:/dev/Omera/scripts/generate-ipc-reference.mjs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
