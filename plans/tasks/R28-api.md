# R28-api / #291: API contracts contradict the implementation status of legacy migration commands

**Assignment:** one issue only. **Package:** [W01](../W01_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Lead contract/documentation engineer | **Verification:** V5.  
**Tracking:** [#291](https://github.com/BerryUIKI/Omera/issues/291) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** None beyond baseline/ownership checks.

**Downstream consumers:** [R05-ui](R05-ui.md) Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Each contract has one unambiguous implementation status and a current runtime identity.

## Recommended implementation

1. Separate implemented IPC contracts from proposed APIs and remove contradictory legacy/current identity statements.
2. Document inputs, outputs, failure semantics, source preservation and status for registered migration commands. Any API change follows repository contract instructions.

## Targeted validation

- [ ] Compare every documented implemented command with registration and frontend/backend DTOs.
- [ ] Confirm a reader cannot mistake a proposed migration API for an invokable runtime command.

## Issue acceptance (preserved verbatim)

- [ ] Remove contradictory blanket status statements.
- [ ] Label proposed and implemented commands individually.
- [ ] Validate DTO compatibility separately from inventory generation.

## Evidence and source pointers

**Code-confirmed documentation contradiction.** API_CONTRACTS opens with unimplemented identity/migration language and legacy runtime identity while a later section declares seven migration commands implemented. The running lib.rs registers those commands. Proposed APIs and implemented migration APIs are not clearly separated.

1. Read implementation-status statements across API_CONTRACTS.
2. Compare each legacy migration command with the actual registration list and DTOs.
3. Identify truly proposed commands separately.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: API contracts contradict the implementation status of legacy migration commands. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [docs/API_CONTRACTS.md](D:/dev/Omera/docs/API_CONTRACTS.md), [src-tauri/src/lib.rs](D:/dev/Omera/src-tauri/src/lib.rs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
