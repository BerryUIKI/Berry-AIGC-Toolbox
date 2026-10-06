# W12 Plan: Optional automation and accurate product capability

**Coordinator:** Backup engineer and documentation engineer; lead scheduler/recovery review  
**Original phase:** 3 | **Cards:** 3

**Outcome:** Scheduled backup has an executable lifecycle; public claims match the actual runtime.

**Original conservative package predecessors:** [W01](W01_PLAN.md), [W03](W03_PLAN.md), [W07](W07_PLAN.md)

This is a package dispatch/index plan, not one Agent's complete assignment. Individual card prerequisites govern dispatch; starting an independent card before an aggregate package gate requires recorded scope/ownership reasoning. Package completion and release qualification retain their broader gates.

## Dispatch order and coordination

Correct R28 and R28-remote-db capability claims during the first wave; they have no task predecessors. R15 scheduler requires R01 safe recovery and R17 accepted configuration. Do not postpone truthful documentation until automatic backup is implemented.

The table is a stable topological order within this package, not an instruction to run every row in one session. External prerequisites remain explicit.

| Task card | Priority | Profile | Task predecessors |
| --- | --- | --- | --- |
| [R15](tasks/R15.md) / #246 - Implement the configured automatic cloud backup scheduler | P2 | V3 | [R01](tasks/R01.md), [R17](tasks/R17.md) |
| [R28](tasks/R28.md) / #257 - README advertises bidirectional cloud sync that the runtime does not implement | P2 | V5 | Baseline/ownership only |
| [R28-remote-db](tasks/R28-remote-db.md) / #289 - README claims shipped MySQL and PostgreSQL storage while runtime is SQLite-only | P2 | V5 | Baseline/ownership only |

## External task prerequisites

The original aggregate package map is not a complete execution graph. Check these concrete cross-package prerequisites even when a package is absent from the original predecessor list.

| Assigned card | External prerequisite | Owning package |
| --- | --- | --- |
| [R15](tasks/R15.md) | [R01](tasks/R01.md) | [W03](W03_PLAN.md) |
| [R15](tasks/R15.md) | [R17](tasks/R17.md) | [W08](W08_PLAN.md) |

## Boundaries and exit

Use each card's exact owner and [runbook](AGENT_RUNBOOK.md). Check actual overlapping files and lead-owned contracts before implementation; consult only relevant rows in [CONTRACTS.md](CONTRACTS.md). Preserve all issue-specific acceptance rather than accepting the package by aggregate test success.

Exit requires integrated-candidate acceptance and evidence for every card in the package. Incomplete GUI/provider/platform checks stay visible in [STATUS.md](STATUS.md); a deferred card is unresolved. See [QUALIFICATION.md](QUALIFICATION.md) for candidate gates.

## Copyable single-card assignment

```text
Repository: D:\dev\Omera. Assigned task: <one ID from W12>.
Read D:\dev\Omera-Review\plans\AGENT_RUNBOOK.md and
D:\dev\Omera-Review\plans\tasks\<ID>.md.
Revalidate current HEAD and integrated task prerequisites.
Implement only this issue and its necessary tests/docs within its owner boundary.
Do not execute the whole package or follow references into new assignments.
Return a concrete handoff with exact commit, acceptance outcomes,
verification results, remaining dependencies and not-run limits.
```

[Coordinator overview](README.md) | [Task index](TASK_INDEX.md) | [Plan review](PLAN_REVIEW.md)
