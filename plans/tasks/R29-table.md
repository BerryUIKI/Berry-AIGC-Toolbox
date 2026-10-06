# R29-table / #263: Make Table columns configurable and usable at normal window widths

**Assignment:** one issue only. **Package:** [W10](../W10_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Gallery/UI engineer | **Verification:** V4.  
**Tracking:** [#263](https://github.com/BerryUIKI/Omera/issues/263) | **Evidence at cutoff:** Proposal.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R30](R30.md)

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior



## Recommended implementation

1. Add usable column sizing/visibility and compatible persisted preferences, with responsive default widths for filenames and prompt/model comparison.
2. Keep controls reachable by keyboard and maintain horizontal scrolling where necessary without layout overflow.

## Targeted validation

- [ ] Verify normal/narrow/wide windows, open/closed sidebars and long filenames/prompts.
- [ ] Test preference migration/reset, column hiding/reordering if exposed, and keyboard/table selection without changing stable gallery card widths.

## Issue acceptance (preserved verbatim)

- [ ] Allow relevant columns to be shown/hidden/resized with accessible controls.
- [ ] Provide a usable compact default at narrow/normal widths and persist user preferences.
- [ ] Verify keyboard navigation, long names, horizontal scrolling and narrow/wide windows.

## Evidence and source pointers

During native inspection at a normal 1280×850 window, the filename column occupied substantial fixed space while prompt/model information required horizontal scrolling. This is a usability proposal; no incorrect pixel rendering or performance regression is claimed.

Reviewed on Omera 0.4.3, `dev` at [`bdac8fb`](https://github.com/BerryUIKI/Omera/commit/bdac8fbcdbac5535075e091453b77c3afe518ba5), Windows development app launched with `pnpm run tauri dev`. Source: [FileList.vue](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/src/components/FileList.vue).

This is the layout/navigation portion of review finding R29, split from localization so it can be discussed and delivered independently. Preserve gallery card-width/virtualization rules and keep all new user text in locale files. Any implementation PR should target `dev`.

**Starting touchpoints:** [src/components/FileList.vue](D:/dev/Omera/src/components/FileList.vue). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
