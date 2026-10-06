# R28-inventory / #292: The checked-in IPC reference does not match the generated command inventory

**Assignment:** one issue only. **Package:** [W01](../W01_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Documentation/QA engineer | **Verification:** V5.  
**Tracking:** [#292](https://github.com/BerryUIKI/Omera/issues/292) | **Evidence at cutoff:** Automated check.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** None beyond baseline/ownership checks.

**Downstream consumers:** [R26-ipc-ci](R26-ipc-ci.md) Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

The committed reference matches the implemented command inventory.

## Recommended implementation

1. Regenerate the IPC reference using the repository script against the fixed baseline and review the generated diff.
2. Repeat regeneration after later command changes; validate DTO compatibility separately and retain compatibility aliases where required.

## Targeted validation

- [ ] Run node scripts/generate-ipc-reference.mjs --check and require a clean result.
- [ ] Review additions/removals against actual registration, without hand-editing generated inventory to conceal drift.

## Issue acceptance (preserved verbatim)

- [ ] Regenerate the reference from current registration.
- [ ] Verify check mode passes without further diff.
- [ ] Review DTO compatibility separately; inventory agreement alone is insufficient.

## Evidence and source pointers

**Reproduced.** node scripts/generate-ipc-reference.mjs --check fails on the reviewed dev baseline. IPC_REFERENCE is stale relative to registered commands, reducing the reliability of the required IPC engineering reference.

1. Check out the reviewed dev baseline.
2. Run node scripts/generate-ipc-reference.mjs --check.
3. Compare the generated inventory with the checked-in reference without changing commands.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: The checked-in IPC reference does not match the generated command inventory. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [scripts/generate-ipc-reference.mjs](D:/dev/Omera/scripts/generate-ipc-reference.mjs), [docs/IPC_REFERENCE.md](D:/dev/Omera/docs/IPC_REFERENCE.md), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
