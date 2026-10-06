# R22-labels / #282: Individual image selection checkboxes have misleading or missing accessible names

**Assignment:** one issue only. **Package:** [W11](../W11_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Accessibility/UI engineer | **Verification:** V4.  
**Tracking:** [#282](https://github.com/BerryUIKI/Omera/issues/282) | **Evidence at cutoff:** Native and source.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R35](R35.md)

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Each selection checkbox has a localized, item-specific accessible name.

## Recommended implementation

1. Give each image checkbox an item-specific localized accessible name and keep the global Select All action distinct.
2. Expose checked/mixed state accurately in Grid/Waterfall/Table while avoiding long/private prompt content in control names.

## Targeted validation

- [ ] Inspect accessibility structure for multiple items and Table rows; require unambiguous names and selection state.
- [ ] Test keyboard toggling and full-query/loaded selection, including mixed exclusion states.

## Issue acceptance (preserved verbatim)

- [ ] Name individual selectors using the image identity.
- [ ] Keep the select-all control distinct from per-item controls.
- [ ] Verify names in all gallery modes.

## Evidence and source pointers

**Code-confirmed and observed accessibility structure.** Grid card checkboxes use the localized Select all label even though each selects one image. Table row checkboxes have no accessible name. Assistive-technology users cannot reliably identify which item a selection control affects.

1. Inspect an individual card checkbox in Grid/Waterfall and a row checkbox in Table.
2. Compare its announced name with the image it selects.
3. Use a keyboard/screen reader to select one image.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Individual image selection checkboxes have misleading or missing accessible names. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src/components/VirtualGrid.vue](D:/dev/Omera/src/components/VirtualGrid.vue), [src/components/FileList.vue](D:/dev/Omera/src/components/FileList.vue), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
