# R26-batch-test / #288: Batch action tests assert copied behavior instead of mounting the production component

**Assignment:** one issue only. **Package:** [W01](../W01_PLAN.md).  
**Proposed priority:** P2 | **Owner:** UI QA engineer | **Verification:** V2.  
**Tracking:** [#288](https://github.com/BerryUIKI/Omera/issues/288) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** None beyond baseline/ownership checks.

**Downstream consumers:** [R26](R26.md) Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Behavioral regression tests exercise the production component or extracted production helpers.

## Recommended implementation

1. Replace copied action/selection implementations in batch-action tests with a mount or equivalent invocation of the actual production Vue component and real emitted events.
2. Mock IPC at its boundary while keeping selection, menu/action wiring and parent handler behavior real. Avoid asserting private implementation details.

## Targeted validation

- [ ] Exercise one/many selections and every exposed batch action's emitted event/payload.
- [ ] Break a production event name or selection update in a temporary branch and confirm the new test catches the regression.

## Issue acceptance (preserved verbatim)

- [ ] Mount BatchActionBar with controlled props and inspect real emitted events.
- [ ] Cover selection changes and keyboard/pointer actions through production code.
- [ ] Keep source-string checks supplemental rather than treating them as interaction tests.

## Evidence and source pointers

**Code-confirmed test-quality gap.** batch-action-bar.test.mjs defines local action/selection behavior and checks those copied functions. The production Vue component is not mounted, so a broken production event handler can coexist with passing tests.

1. Compare test-local actions/selectors with BatchActionBar.vue.
2. Break an actual emitted event in an isolated validation branch.
3. Confirm whether the current assertions exercise that handler.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Batch action tests assert copied behavior instead of mounting the production component. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [tests/batch-action-bar.test.mjs](D:/dev/Omera/tests/batch-action-bar.test.mjs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
