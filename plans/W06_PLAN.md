# W06 Plan: Reliable discovery and folder ingestion

**Coordinator:** Scan engineer; UI engineer for onboarding; lead reconciliation review  
**Original phase:** 2 | **Cards:** 3

**Outcome:** Partial scans preserve records; onboarding indexes existing files; pipeline monitoring consumes its actual source.

**Original conservative package predecessors:** [W01](W01_PLAN.md), [W04](W04_PLAN.md)

This is a package dispatch/index plan, not one Agent's complete assignment. Individual card prerequisites govern dispatch; starting an independent card before an aggregate package gate requires recorded scope/ownership reasoning. Package completion and release qualification retain their broader gates.

## Dispatch order and coordination

R12 scan completeness is an independent P1 after baseline/ownership checks; R34 onboarding follows it. R16 watcher activation waits for the safe pipeline publication/cleanup cards. Do not enable automatic ingestion before those safety invariants hold.

The table is a stable topological order within this package, not an instruction to run every row in one session. External prerequisites remain explicit.

| Task card | Priority | Profile | Task predecessors |
| --- | --- | --- | --- |
| [R12](tasks/R12.md) / #243 - An incomplete filesystem walk is treated as proof that indexed files disappeared | P1 | V1 | Baseline/ownership only |
| [R16](tasks/R16.md) / #247 - Real-time pipeline ingestion is configured but never triggered | P2 | V3 | [R33-pipeline](tasks/R33-pipeline.md), [R02](tasks/R02.md), [R03](tasks/R03.md) |
| [R34](tasks/R34.md) / #266 - Adding a populated folder does not start its initial scan or populate the gallery | P2 | V4 | [R12](tasks/R12.md) |

## External task prerequisites

The original aggregate package map is not a complete execution graph. Check these concrete cross-package prerequisites even when a package is absent from the original predecessor list.

| Assigned card | External prerequisite | Owning package |
| --- | --- | --- |
| [R16](tasks/R16.md) | [R33-pipeline](tasks/R33-pipeline.md) | [W04](W04_PLAN.md) |
| [R16](tasks/R16.md) | [R02](tasks/R02.md) | [W04](W04_PLAN.md) |
| [R16](tasks/R16.md) | [R03](tasks/R03.md) | [W04](W04_PLAN.md) |

## Boundaries and exit

Use each card's exact owner and [runbook](AGENT_RUNBOOK.md). Check actual overlapping files and lead-owned contracts before implementation; consult only relevant rows in [CONTRACTS.md](CONTRACTS.md). Preserve all issue-specific acceptance rather than accepting the package by aggregate test success.

Exit requires integrated-candidate acceptance and evidence for every card in the package. Incomplete GUI/provider/platform checks stay visible in [STATUS.md](STATUS.md); a deferred card is unresolved. See [QUALIFICATION.md](QUALIFICATION.md) for candidate gates.

## Copyable single-card assignment

```text
Repository: D:\dev\Omera. Assigned task: <one ID from W06>.
Read D:\dev\Omera-Review\plans\AGENT_RUNBOOK.md and
D:\dev\Omera-Review\plans\tasks\<ID>.md.
Revalidate current HEAD and integrated task prerequisites.
Implement only this issue and its necessary tests/docs within its owner boundary.
Do not execute the whole package or follow references into new assignments.
Return a concrete handoff with exact commit, acceptance outcomes,
verification results, remaining dependencies and not-run limits.
```

[Coordinator overview](README.md) | [Task index](TASK_INDEX.md) | [Plan review](PLAN_REVIEW.md)
