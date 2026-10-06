# W10 Plan: Gallery semantics and masking controls

**Coordinator:** Gallery/UI engineer; lead review of query-selection/storage contracts  
**Original phase:** 3 | **Cards:** 5

**Outcome:** View-independent masking, complete stacks, explicit selection scope and usable Table controls.

**Original conservative package predecessors:** [W01](W01_PLAN.md), [W05](W05_PLAN.md), [W06](W06_PLAN.md), [W09](W09_PLAN.md)

This is a package dispatch/index plan, not one Agent's complete assignment. Individual card prerequisites govern dispatch; starting an independent card before an aggregate package gate requires recorded scope/ownership reasoning. Package completion and release qualification retain their broader gates.

## Dispatch order and coordination

R21 Table masking is independent after baseline/ownership checks. Stack/selection/table work follows R19/R30 as listed. R37 keeps its recorded R21/R36 prerequisites, although classification and presentation are distinct; the lead can review whether that edge is a completion gate instead of a start blocker. No edge is removed here.

The table is a stable topological order within this package, not an instruction to run every row in one session. External prerequisites remain explicit.

| Task card | Priority | Profile | Task predecessors |
| --- | --- | --- | --- |
| [R21](tasks/R21.md) / #252 - Table mode bypasses the NSFW blur preference | P2 | V4 | Baseline/ownership only |
| [R30](tasks/R30.md) / #259 - Table mode hides collapsed stack members without an expansion affordance | P2 | V4 | [R19](tasks/R19.md) |
| [R35](tasks/R35.md) / #295 - Select All silently selects only the loaded page in a larger gallery | P2 | V4 | [R19](tasks/R19.md), [R30](tasks/R30.md) |
| [R29-table](tasks/R29-table.md) / #263 - Make Table columns configurable and usable at normal window widths | P2 | V4 | [R30](tasks/R30.md) |
| [R37](tasks/R37.md) / #297 - Add a main-toolbar toggle for sensitive-content masking | P2 | V4 | [R21](tasks/R21.md), [R36](tasks/R36.md) |

## External task prerequisites

The original aggregate package map is not a complete execution graph. Check these concrete cross-package prerequisites even when a package is absent from the original predecessor list.

| Assigned card | External prerequisite | Owning package |
| --- | --- | --- |
| [R30](tasks/R30.md) | [R19](tasks/R19.md) | [W09](W09_PLAN.md) |
| [R35](tasks/R35.md) | [R19](tasks/R19.md) | [W09](W09_PLAN.md) |
| [R37](tasks/R37.md) | [R36](tasks/R36.md) | [W05](W05_PLAN.md) |

## Boundaries and exit

Use each card's exact owner and [runbook](AGENT_RUNBOOK.md). Check actual overlapping files and lead-owned contracts before implementation; consult only relevant rows in [CONTRACTS.md](CONTRACTS.md). Preserve all issue-specific acceptance rather than accepting the package by aggregate test success.

Exit requires integrated-candidate acceptance and evidence for every card in the package. Incomplete GUI/provider/platform checks stay visible in [STATUS.md](STATUS.md); a deferred card is unresolved. See [QUALIFICATION.md](QUALIFICATION.md) for candidate gates.

## Copyable single-card assignment

```text
Repository: D:\dev\Omera. Assigned task: <one ID from W10>.
Read D:\dev\Omera-Review\plans\AGENT_RUNBOOK.md and
D:\dev\Omera-Review\plans\tasks\<ID>.md.
Revalidate current HEAD and integrated task prerequisites.
Implement only this issue and its necessary tests/docs within its owner boundary.
Do not execute the whole package or follow references into new assignments.
Return a concrete handoff with exact commit, acceptance outcomes,
verification results, remaining dependencies and not-run limits.
```

[Coordinator overview](README.md) | [Task index](TASK_INDEX.md) | [Plan review](PLAN_REVIEW.md)
