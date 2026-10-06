# R29-tags / #264: Add search or filtering to the Sidebar tag list

**Assignment:** one issue only. **Package:** [W11](../W11_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Sidebar/UI engineer | **Verification:** V2.  
**Tracking:** [#264](https://github.com/BerryUIKI/Omera/issues/264) | **Evidence at cutoff:** Proposal.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R22-sidebar](R22-sidebar.md)

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior



## Recommended implementation

1. Add direct tag search/filter with localized empty/loading state and keyboard-operable results.
2. Keep rendering/query work bounded and preserve the active tag selection while the filter changes; avoid mutating tags through a search control.

## Targeted validation

- [ ] Test a long tag set, case/Unicode search, no matches, clearing filter and keyboard activation.
- [ ] Verify stable selected counts and usability at narrow widths without mounting every library image.

## Issue acceptance (preserved verbatim)

- [ ] Provide a localized, keyboard-operable tag search/filter control.
- [ ] Preserve selected tags and make empty/no-match states clear.
- [ ] Keep rendering/query work bounded for large tag collections and test rapid input.

## Evidence and source pointers

The current Sidebar exposes a long alphabetical tag list without a direct tag search/filter affordance. Finding a particular tag requires scanning or scrolling. This is a usability proposal observed during native inspection.

Reviewed on Omera 0.4.3, `dev` at [`bdac8fb`](https://github.com/BerryUIKI/Omera/commit/bdac8fbcdbac5535075e091453b77c3afe518ba5), Windows development app launched with `pnpm run tauri dev`. Source: [Sidebar.vue](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/src/components/Sidebar.vue).

This is the layout/navigation portion of review finding R29, split from localization so it can be discussed and delivered independently. Preserve gallery card-width/virtualization rules and keep all new user text in locale files. Any implementation PR should target `dev`.

**Starting touchpoints:** [src/components/Sidebar.vue](D:/dev/Omera/src/components/Sidebar.vue). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
