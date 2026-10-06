# R18-concurrency / #279: ActionHistory permits overlapping undo transitions

**Assignment:** one issue only. **Package:** [W09](../W09_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Frontend state engineer | **Verification:** V2.  
**Tracking:** [#279](https://github.com/BerryUIKI/Omera/issues/279) | **Evidence at cutoff:** Production reproduction.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R18](R18.md)

**Downstream consumers:** [R19](R19.md) Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

History transitions have a defined serial ordering or explicitly reject concurrent transitions.

## Recommended implementation

1. Serialize execute/undo/redo transitions or enforce one well-defined pending-operation guard.
2. Make rapid shortcuts/menu actions respect pending state and complete in a deterministic order; preserve failure retry behavior.

## Targeted validation

- [ ] Re-run the barrier test and require peak simultaneous transition callbacks of one.
- [ ] Test rapid undo/redo/execute combinations, rejection and reentrancy so history order and state remain coherent.

## Issue acceptance (preserved verbatim)

- [ ] Guard or serialize execute/undo/redo while a transition is pending.
- [ ] Preserve the ordering contract on completion and failure.
- [ ] Test overlapping transitions without timers or live IPC.

## Evidence and source pointers

**Reproduced using a controlled barrier and the production class.** Two undo calls execute concurrently; the peak in-flight undo count is 2 instead of 1. History has no pending-operation guard or serialization contract, allowing rapid shortcuts to apply state-dependent commands out of order.

1. Execute two commands with undo callbacks paused by independently controlled barriers.
2. Call undo twice before releasing either barrier.
3. Record peak in-flight callbacks, release them in reverse order and inspect history.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: ActionHistory permits overlapping undo transitions. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src/utils/history.ts](D:/dev/Omera/src/utils/history.ts), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
