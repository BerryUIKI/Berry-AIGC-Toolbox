# W02 Plan: Persistent record identity

**Coordinator:** Lead storage engineer  
**Original phase:** 1 | **Cards:** 1

**Outcome:** Every insert/upsert returns the identity of the actual affected row.

**Original conservative package predecessors:** [W01](W01_PLAN.md)

This is a package dispatch/index plan, not one Agent's complete assignment. Individual card prerequisites govern dispatch; starting an independent card before an aggregate package gate requires recorded scope/ownership reasoning. Package completion and release qualification retain their broader gates.

## Dispatch order and coordination

R11 follows the isolated formatting correction. Verify every ID-consuming caller; this is a lead storage assignment. Do not wait for unrelated W01 documentation before reproducing the row-identity failure.

The table is a stable topological order within this package, not an instruction to run every row in one session. External prerequisites remain explicit.

| Task card | Priority | Profile | Task predecessors |
| --- | --- | --- | --- |
| [R11](tasks/R11.md) / #241 - Upserting an existing file returns an unrelated file's ID | P1 | V1 | [R26-format](tasks/R26-format.md) |

## External task prerequisites

The original aggregate package map is not a complete execution graph. Check these concrete cross-package prerequisites even when a package is absent from the original predecessor list.

| Assigned card | External prerequisite | Owning package |
| --- | --- | --- |
| [R11](tasks/R11.md) | [R26-format](tasks/R26-format.md) | [W01](W01_PLAN.md) |

## Boundaries and exit

Use each card's exact owner and [runbook](AGENT_RUNBOOK.md). Check actual overlapping files and lead-owned contracts before implementation; consult only relevant rows in [CONTRACTS.md](CONTRACTS.md). Preserve all issue-specific acceptance rather than accepting the package by aggregate test success.

Exit requires integrated-candidate acceptance and evidence for every card in the package. Incomplete GUI/provider/platform checks stay visible in [STATUS.md](STATUS.md); a deferred card is unresolved. See [QUALIFICATION.md](QUALIFICATION.md) for candidate gates.

## Copyable single-card assignment

```text
Repository: D:\dev\Omera. Assigned task: <one ID from W02>.
Read D:\dev\Omera-Review\plans\AGENT_RUNBOOK.md and
D:\dev\Omera-Review\plans\tasks\<ID>.md.
Revalidate current HEAD and integrated task prerequisites.
Implement only this issue and its necessary tests/docs within its owner boundary.
Do not execute the whole package or follow references into new assignments.
Return a concrete handoff with exact commit, acceptance outcomes,
verification results, remaining dependencies and not-run limits.
```

[Coordinator overview](README.md) | [Task index](TASK_INDEX.md) | [Plan review](PLAN_REVIEW.md)
