# R17-mirror / #277: Failed configuration saves overwrite the localStorage mirror

**Assignment:** one issue only. **Package:** [W08](../W08_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Configuration engineer; lead revision review | **Verification:** V3.  
**Tracking:** [#277](https://github.com/BerryUIKI/Omera/issues/277) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** None beyond baseline/ownership checks.

**Downstream consumers:** [R17](R17.md) Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

The compatibility mirror reflects successfully persisted configuration.

## Recommended implementation

1. Update localStorage and active configuration mirrors only after the backend accepts and persists the values.
2. On rejection, preserve the last accepted mirror and surface a structured error; maintain compatible legacy key migration.

## Targeted validation

- [ ] Reject a backend save and require all mirrors/readers to retain accepted values.
- [ ] Test successful save, restart, partial/retry scenarios and legacy localStorage migration.

## Issue acceptance (preserved verbatim)

- [ ] Update the mirror after confirmed backend success.
- [ ] Preserve the previous accepted mirror on rejection.
- [ ] Test backend failure and successful persistence/mirror consistency.

## Evidence and source pointers

**Code-confirmed.** saveAppConfig catches a backend save failure, writes the rejected values into localStorage via syncConfigToLocalStorage, and then rethrows. Consumers can read a configuration that was never durably accepted by the backend.

1. Seed a known successful config mirror in an isolated localStorage fixture.
2. Make the backend save reject while saving different values.
3. Read the mirror and compare with the last accepted backend config.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Failed configuration saves overwrite the localStorage mirror. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src/utils/config.ts](D:/dev/Omera/src/utils/config.ts), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
