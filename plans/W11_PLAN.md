# W11 Plan: Statistics, accessibility, locales and tag discovery

**Coordinator:** UI/accessibility engineer; lead review for new bounded statistics queries  
**Original phase:** 3 | **Cards:** 7

**Outcome:** Truthful statistical populations, keyboard-operable controls, localized feedback and direct tag lookup.

**Original conservative package predecessors:** [W01](W01_PLAN.md), [W02](W02_PLAN.md)

This is a package dispatch/index plan, not one Agent's complete assignment. Individual card prerequisites govern dispatch; starting an independent card before an aggregate package gate requires recorded scope/ownership reasoning. Package completion and release qualification retain their broader gates.

## Dispatch order and coordination

Statistics, menus, Sidebar keyboard access, labels, localization and tag search are distinct lanes. R20, R22 and R22-sidebar have no task predecessors. R22-labels waits for R35 and R29 waits for config/history prerequisites. Keep Sidebar.vue and locale edits coordinated.

The table is a stable topological order within this package, not an instruction to run every row in one session. External prerequisites remain explicit.

| Task card | Priority | Profile | Task predecessors |
| --- | --- | --- | --- |
| [R20](tasks/R20.md) / #251 - Model and sampler statistics tabs receive hardcoded empty arrays | P2 | V3 | Baseline/ownership only |
| [R22](tasks/R22.md) / #253 - Application menus lack keyboard dismissal and focus navigation | P2 | V2 | Baseline/ownership only |
| [R22-sidebar](tasks/R22-sidebar.md) / #281 - Sidebar destinations and tags are not keyboard-operable | P2 | V2 | Baseline/ownership only |
| [R22-labels](tasks/R22-labels.md) / #282 - Individual image selection checkboxes have misleading or missing accessible names | P2 | V4 | [R35](tasks/R35.md) |
| [R29](tasks/R29.md) / #258 - Translate hardcoded statistics, history and Sidebar text into the selected locale | P2 | V2 | [R17](tasks/R17.md), [R18](tasks/R18.md) |
| [R20-denominator](tasks/R20-denominator.md) / #280 - Prompt statistics counts all indexed files as analyzed | P2 | V3 | [R20](tasks/R20.md) |
| [R29-tags](tasks/R29-tags.md) / #264 - Add search or filtering to the Sidebar tag list | P2 | V2 | [R22-sidebar](tasks/R22-sidebar.md) |

## External task prerequisites

The original aggregate package map is not a complete execution graph. Check these concrete cross-package prerequisites even when a package is absent from the original predecessor list.

| Assigned card | External prerequisite | Owning package |
| --- | --- | --- |
| [R22-labels](tasks/R22-labels.md) | [R35](tasks/R35.md) | [W10](W10_PLAN.md) |
| [R29](tasks/R29.md) | [R17](tasks/R17.md) | [W08](W08_PLAN.md) |
| [R29](tasks/R29.md) | [R18](tasks/R18.md) | [W09](W09_PLAN.md) |

## Boundaries and exit

Use each card's exact owner and [runbook](AGENT_RUNBOOK.md). Check actual overlapping files and lead-owned contracts before implementation; consult only relevant rows in [CONTRACTS.md](CONTRACTS.md). Preserve all issue-specific acceptance rather than accepting the package by aggregate test success.

Exit requires integrated-candidate acceptance and evidence for every card in the package. Incomplete GUI/provider/platform checks stay visible in [STATUS.md](STATUS.md); a deferred card is unresolved. See [QUALIFICATION.md](QUALIFICATION.md) for candidate gates.

## Copyable single-card assignment

```text
Repository: D:\dev\Omera. Assigned task: <one ID from W11>.
Read D:\dev\Omera-Review\plans\AGENT_RUNBOOK.md and
D:\dev\Omera-Review\plans\tasks\<ID>.md.
Revalidate current HEAD and integrated task prerequisites.
Implement only this issue and its necessary tests/docs within its owner boundary.
Do not execute the whole package or follow references into new assignments.
Return a concrete handoff with exact commit, acceptance outcomes,
verification results, remaining dependencies and not-run limits.
```

[Coordinator overview](README.md) | [Task index](TASK_INDEX.md) | [Plan review](PLAN_REVIEW.md)
