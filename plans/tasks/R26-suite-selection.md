# R26-suite-selection / #287: The frontend test entry point excludes four existing regression suites

**Assignment:** one issue only. **Package:** [W01](../W01_PLAN.md).  
**Proposed priority:** P2 | **Owner:** QA engineer | **Verification:** V2.  
**Tracking:** [#287](https://github.com/BerryUIKI/Omera/issues/287) | **Evidence at cutoff:** Automated check.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** None beyond baseline/ownership checks.

**Downstream consumers:** [R26](R26.md) Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

The maintained frontend entry point executes every intended regression suite.

## Recommended implementation

1. Replace or extend the explicit test-file selection so the four omitted maintained suites are included. Prefer a reliable discovery pattern or a checked list with a guard against accidental omissions.
2. Keep distinct scripts only where their scope is documented; do not silently substitute a smaller smoke suite for the full frontend regression run.

## Targeted validation

- [ ] Confirm autotag-mount, autotag-ui-distinct-actions, folder-removal-confirm and settings-storage-backend all run.
- [ ] Verify test discovery rejects a misspelled/missing maintained file and reports the executed test count.

## Issue acceptance (preserved verbatim)

- [ ] Include the four suites or use a controlled discovery convention.
- [ ] Document intentional exclusions.
- [ ] Test that new maintained suites are not silently skipped.

## Evidence and source pointers

**Reproduced by script inspection and separate execution.** test:stack uses an explicit file list that omits autotag-mount, autotag-ui-distinct-actions, folder-removal-confirm and settings-storage-backend. Their seven tests passed when run separately, but the normal entry point does not execute them.

1. Compare test:stack with the maintained files under tests/.
2. Run the four omitted suites separately.
3. Use a failing assertion in one omitted suite to verify that test:stack currently cannot detect it.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: The frontend test entry point excludes four existing regression suites. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [package.json](D:/dev/Omera/package.json), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
