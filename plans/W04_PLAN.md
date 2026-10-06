# W04 Plan: Shared file publication and source disposition

**Coordinator:** Lead filesystem engineer  
**Original phase:** 1 | **Cards:** 8

**Outcome:** No-clobber publication, verified identity, honest receipts and recoverable disposition across callers.

**Original conservative package predecessors:** [W01](W01_PLAN.md), [W02](W02_PLAN.md)

This is a package dispatch/index plan, not one Agent's complete assignment. Individual card prerequisites govern dispatch; starting an independent card before an aggregate package gate requires recorded scope/ownership reasoning. Package completion and release qualification retain their broader gates.

## Dispatch order and coordination

R11 -> R33 -> R33-pipeline -> R02 -> R03 is the main publication/cleanup lane. Other cards branch from the explicitly listed task prerequisites. A large extraction need not be the only safety strategy: the lead may propose a bounded production-path fix, with an explicit dependency revision and unchanged issue acceptance. This split does not approve or apply that revision.

The table is a stable topological order within this package, not an instruction to run every row in one session. External prerequisites remain explicit.

| Task card | Priority | Profile | Task predecessors |
| --- | --- | --- | --- |
| [R33](tasks/R33.md) / #262 - Managed import duplicates publication logic across IPC and reusable services | P2 | V3 | [R11](tasks/R11.md) |
| [R33-pipeline](tasks/R33-pipeline.md) / #293 - Extract pipeline harvesting and cleanup business logic from Tauri command adapters | P2 | V3 | [R33](tasks/R33.md) |
| [R31](tasks/R31.md) / #260 - Reusable managed-transform import accepts a linked destination | P2 | V1 | [R33](tasks/R33.md) |
| [R08](tasks/R08.md) / #238 - Batch transform archives or trashes the source before database persistence succeeds | P1 | V1 | [R33](tasks/R33.md) |
| [R32](tasks/R32.md) / #261 - Registered cull command counts failed trash operations as successful | P2 | V1 | [R33](tasks/R33.md) |
| [R02](tasks/R02.md) / #232 - Pipeline harvesting overwrites existing assets on filename collisions | P1 | V1 | [R33-pipeline](tasks/R33-pipeline.md) |
| [R02-dedup](tasks/R02-dedup.md) / #268 - Managed import treats equal byte lengths as identical content | P1 | V1 | [R33](tasks/R33.md), [R33-pipeline](tasks/R33-pipeline.md) |
| [R03](tasks/R03.md) / #233 - Deferred pipeline cleanup does not revalidate the destination or source identity | P1 | V1 | [R11](tasks/R11.md), [R02](tasks/R02.md) |

## External task prerequisites

The original aggregate package map is not a complete execution graph. Check these concrete cross-package prerequisites even when a package is absent from the original predecessor list.

| Assigned card | External prerequisite | Owning package |
| --- | --- | --- |
| [R33](tasks/R33.md) | [R11](tasks/R11.md) | [W02](W02_PLAN.md) |
| [R03](tasks/R03.md) | [R11](tasks/R11.md) | [W02](W02_PLAN.md) |

## Boundaries and exit

Use each card's exact owner and [runbook](AGENT_RUNBOOK.md). Check actual overlapping files and lead-owned contracts before implementation; consult only relevant rows in [CONTRACTS.md](CONTRACTS.md). Preserve all issue-specific acceptance rather than accepting the package by aggregate test success.

Exit requires integrated-candidate acceptance and evidence for every card in the package. Incomplete GUI/provider/platform checks stay visible in [STATUS.md](STATUS.md); a deferred card is unresolved. See [QUALIFICATION.md](QUALIFICATION.md) for candidate gates.

## Copyable single-card assignment

```text
Repository: D:\dev\Omera. Assigned task: <one ID from W04>.
Read D:\dev\Omera-Review\plans\AGENT_RUNBOOK.md and
D:\dev\Omera-Review\plans\tasks\<ID>.md.
Revalidate current HEAD and integrated task prerequisites.
Implement only this issue and its necessary tests/docs within its owner boundary.
Do not execute the whole package or follow references into new assignments.
Return a concrete handoff with exact commit, acceptance outcomes,
verification results, remaining dependencies and not-run limits.
```

[Coordinator overview](README.md) | [Task index](TASK_INDEX.md) | [Plan review](PLAN_REVIEW.md)
