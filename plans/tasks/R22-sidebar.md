# R22-sidebar / #281: Sidebar destinations and tags are not keyboard-operable

**Assignment:** one issue only. **Package:** [W11](../W11_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Accessibility/UI engineer | **Verification:** V2.  
**Tracking:** [#281](https://github.com/BerryUIKI/Omera/issues/281) | **Evidence at cutoff:** Native and source.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** None beyond baseline/ownership checks.

**Downstream consumers:** [R29-tags](R29-tags.md) Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Every Sidebar navigation action is reachable and activatable by keyboard.

## Recommended implementation

1. Use semantic interactive navigation controls or an equivalent roving-focus contract for Sidebar destinations and tags.
2. Preserve selected/current state, visible focus and keyboard activation without recursively mounting every row.

## Targeted validation

- [ ] Navigate folders, destinations and tags with keyboard and assistive names/states.
- [ ] Test deep/collapsed trees, long lists, empty nodes and focus retention after updates.

## Issue acceptance (preserved verbatim)

- [ ] Use focusable controls or implement the intended navigation widget pattern.
- [ ] Provide Enter/Space activation and visible focus.
- [ ] Test destinations, nested folders and tag navigation while preserving input/modal shortcuts.

## Evidence and source pointers

**Code-confirmed and observed accessibility structure.** Sidebar destinations and tag groups are click-driven list/group elements without an equivalent keyboard activation path. Keyboard users cannot consistently reach and activate the same navigation destinations as pointer users.

1. Tab through the sidebar using only the keyboard.
2. Attempt to activate a library destination, folder and tag without a pointer.
3. Inspect focusability and selected/current semantics.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Sidebar destinations and tags are not keyboard-operable. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src/components/Sidebar.vue](D:/dev/Omera/src/components/Sidebar.vue), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
