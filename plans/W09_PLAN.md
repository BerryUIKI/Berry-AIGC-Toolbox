# W09 Plan: History serialization and mutation refresh

**Coordinator:** Frontend state engineer  
**Original phase:** 2 | **Cards:** 3

**Outcome:** Rejected history transitions remain retryable; concurrent transitions serialize; queries/counters update coherently.

**Original conservative package predecessors:** [W01](W01_PLAN.md)

This is a package dispatch/index plan, not one Agent's complete assignment. Individual card prerequisites govern dispatch; starting an independent card before an aggregate package gate requires recorded scope/ownership reasoning. Package completion and release qualification retain their broader gates.

## Dispatch order and coordination

R18 -> R18-concurrency -> R19. The first two share history.ts and should be consecutive focused changes. R19 then updates App.vue mutation refresh and requires full gallery verification.

The table is a stable topological order within this package, not an instruction to run every row in one session. External prerequisites remain explicit.

| Task card | Priority | Profile | Task predecessors |
| --- | --- | --- | --- |
| [R18](tasks/R18.md) / #249 - Failed undo or redo removes the command from history | P2 | V2 | Baseline/ownership only |
| [R18-concurrency](tasks/R18-concurrency.md) / #279 - ActionHistory permits overlapping undo transitions | P2 | V2 | [R18](tasks/R18.md) |
| [R19](tasks/R19.md) / #250 - Batch favorite/NSFW changes and their undo leave filters and counters stale | P2 | V4 | [R18](tasks/R18.md), [R18-concurrency](tasks/R18-concurrency.md) |

## External task prerequisites

The original aggregate package map is not a complete execution graph. Check these concrete cross-package prerequisites even when a package is absent from the original predecessor list.

No external task edge is recorded; baseline, ownership and relevant contract checks still apply.

## Boundaries and exit

Use each card's exact owner and [runbook](AGENT_RUNBOOK.md). Check actual overlapping files and lead-owned contracts before implementation; consult only relevant rows in [CONTRACTS.md](CONTRACTS.md). Preserve all issue-specific acceptance rather than accepting the package by aggregate test success.

Exit requires integrated-candidate acceptance and evidence for every card in the package. Incomplete GUI/provider/platform checks stay visible in [STATUS.md](STATUS.md); a deferred card is unresolved. See [QUALIFICATION.md](QUALIFICATION.md) for candidate gates.

## Copyable single-card assignment

```text
Repository: D:\dev\Omera. Assigned task: <one ID from W09>.
Read D:\dev\Omera-Review\plans\AGENT_RUNBOOK.md and
D:\dev\Omera-Review\plans\tasks\<ID>.md.
Revalidate current HEAD and integrated task prerequisites.
Implement only this issue and its necessary tests/docs within its owner boundary.
Do not execute the whole package or follow references into new assignments.
Return a concrete handoff with exact commit, acceptance outcomes,
verification results, remaining dependencies and not-run limits.
```

[Coordinator overview](README.md) | [Task index](TASK_INDEX.md) | [Plan review](PLAN_REVIEW.md)
