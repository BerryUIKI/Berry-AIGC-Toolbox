# W08 Plan: Authoritative configuration transactions

**Coordinator:** Configuration/UI engineer with lead concurrency review  
**Original phase:** 2 | **Cards:** 3

**Outcome:** Only accepted durable values become active/mirrored; stale edits conflict and failed saves remain retryable.

**Original conservative package predecessors:** [W01](W01_PLAN.md)

This is a package dispatch/index plan, not one Agent's complete assignment. Individual card prerequisites govern dispatch; starting an independent card before an aggregate package gate requires recorded scope/ownership reasoning. Package completion and release qualification retain their broader gates.

## Dispatch order and coordination

R17-mirror and R17-revision establish authoritative accepted state; R17 UI follows both. They have different starting files, but one transaction/revision contract. Verify failed and stale saves before wiring success feedback.

The table is a stable topological order within this package, not an instruction to run every row in one session. External prerequisites remain explicit.

| Task card | Priority | Profile | Task predecessors |
| --- | --- | --- | --- |
| [R17-mirror](tasks/R17-mirror.md) / #277 - Failed configuration saves overwrite the localStorage mirror | P2 | V3 | Baseline/ownership only |
| [R17-revision](tasks/R17-revision.md) / #278 - Settings save reloads the latest revision before overwriting stale form values | P2 | V3 | Baseline/ownership only |
| [R17](tasks/R17.md) / #248 - Settings dialog reports success and closes after a failed save | P2 | V2 | [R17-mirror](tasks/R17-mirror.md), [R17-revision](tasks/R17-revision.md) |

## External task prerequisites

The original aggregate package map is not a complete execution graph. Check these concrete cross-package prerequisites even when a package is absent from the original predecessor list.

No external task edge is recorded; baseline, ownership and relevant contract checks still apply.

## Boundaries and exit

Use each card's exact owner and [runbook](AGENT_RUNBOOK.md). Check actual overlapping files and lead-owned contracts before implementation; consult only relevant rows in [CONTRACTS.md](CONTRACTS.md). Preserve all issue-specific acceptance rather than accepting the package by aggregate test success.

Exit requires integrated-candidate acceptance and evidence for every card in the package. Incomplete GUI/provider/platform checks stay visible in [STATUS.md](STATUS.md); a deferred card is unresolved. See [QUALIFICATION.md](QUALIFICATION.md) for candidate gates.

## Copyable single-card assignment

```text
Repository: D:\dev\Omera. Assigned task: <one ID from W08>.
Read D:\dev\Omera-Review\plans\AGENT_RUNBOOK.md and
D:\dev\Omera-Review\plans\tasks\<ID>.md.
Revalidate current HEAD and integrated task prerequisites.
Implement only this issue and its necessary tests/docs within its owner boundary.
Do not execute the whole package or follow references into new assignments.
Return a concrete handoff with exact commit, acceptance outcomes,
verification results, remaining dependencies and not-run limits.
```

[Coordinator overview](README.md) | [Task index](TASK_INDEX.md) | [Plan review](PLAN_REVIEW.md)
