# R17-revision / #278: Settings save reloads the latest revision before overwriting stale form values

**Assignment:** one issue only. **Package:** [W08](../W08_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Configuration/UI engineer; lead revision review | **Verification:** V3.  
**Tracking:** [#278](https://github.com/BerryUIKI/Omera/issues/278) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** None beyond baseline/ownership checks.

**Downstream consumers:** [R17](R17.md) Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

The save uses the form's original revision or explicitly resolves concurrent changes.

## Recommended implementation

1. Capture the configuration revision when loading the form and submit that revision with the user's edits.
2. Handle conflict explicitly by retaining edits and offering reconcile/reload, rather than reloading the latest revision to disguise stale values.

## Targeted validation

- [ ] Edit from two forms/consumers and require the stale save to conflict without overwriting newer data.
- [ ] Test nonoverlapping changes under the agreed merge policy, retry and legacy default compatibility.

## Issue acceptance (preserved verbatim)

- [ ] Capture the loaded form revision.
- [ ] Detect conflicting edits rather than silently borrowing a newer revision.
- [ ] Test two writers, non-overlapping changes and explicit conflict recovery.

## Evidence and source pointers

**Code-confirmed.** The dialog reloads the latest configuration revision immediately before saving, then overlays values from an older form. This bypasses detection of concurrent edits to those fields: stale form data is written with the newer revision.

1. Open form A at revision N in a controlled configuration fixture.
2. Commit an overlapping field change from writer B, producing revision N+1.
3. Save unchanged/stale form A and inspect whether the conflict is surfaced.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Settings save reloads the latest revision before overwriting stale form values. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src/components/SettingsModal.vue](D:/dev/Omera/src/components/SettingsModal.vue), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
