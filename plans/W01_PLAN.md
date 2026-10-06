# W01 Plan: Baseline, regression infrastructure and engineering truth

**Coordinator:** QA engineer with lead review for contract documents  
**Original phase:** 0 | **Cards:** 8

**Outcome:** A reproducible candidate baseline, production-path tests in CI, and accurate engineering references.

**Original conservative package predecessors:** None.

This is a package dispatch/index plan, not one Agent's complete assignment. Individual card prerequisites govern dispatch; starting an independent card before an aggregate package gate requires recorded scope/ownership reasoning. Package completion and release qualification retain their broader gates.

## Dispatch order and coordination

Record the candidate first. R26-format, R26-suite-selection, R26-batch-test and the three R28 engineering-reference cards have no task predecessors. R26 follows suite selection and component tests; R26-ipc-ci follows R28-inventory. Coordinate the two CI edits and generator/reference edits.

The table is a stable topological order within this package, not an instruction to run every row in one session. External prerequisites remain explicit.

| Task card | Priority | Profile | Task predecessors |
| --- | --- | --- | --- |
| [R26-suite-selection](tasks/R26-suite-selection.md) / #287 - The frontend test entry point excludes four existing regression suites | P2 | V2 | Baseline/ownership only |
| [R26-batch-test](tasks/R26-batch-test.md) / #288 - Batch action tests assert copied behavior instead of mounting the production component | P2 | V2 | Baseline/ownership only |
| [R26-format](tasks/R26-format.md) / #294 - Existing Rust formatting drift fails CI on every platform | P2 | V0 | Baseline/ownership only |
| [R28-handoff](tasks/R28-handoff.md) / #290 - Engineering handoff reports an outdated identity migration status | P2 | V5 | Baseline/ownership only |
| [R28-api](tasks/R28-api.md) / #291 - API contracts contradict the implementation status of legacy migration commands | P2 | V5 | Baseline/ownership only |
| [R28-inventory](tasks/R28-inventory.md) / #292 - The checked-in IPC reference does not match the generated command inventory | P2 | V5 | Baseline/ownership only |
| [R26](tasks/R26.md) / #108 - CI does not run the maintained frontend regression suites | P2 | V2 | [R26-suite-selection](tasks/R26-suite-selection.md), [R26-batch-test](tasks/R26-batch-test.md) |
| [R26-ipc-ci](tasks/R26-ipc-ci.md) / #286 - CI does not check the generated IPC inventory for drift | P2 | V5 | [R28-inventory](tasks/R28-inventory.md) |

## External task prerequisites

The original aggregate package map is not a complete execution graph. Check these concrete cross-package prerequisites even when a package is absent from the original predecessor list.

No external task edge is recorded; baseline, ownership and relevant contract checks still apply.

## Boundaries and exit

Use each card's exact owner and [runbook](AGENT_RUNBOOK.md). Check actual overlapping files and lead-owned contracts before implementation; consult only relevant rows in [CONTRACTS.md](CONTRACTS.md). Preserve all issue-specific acceptance rather than accepting the package by aggregate test success.

Exit requires integrated-candidate acceptance and evidence for every card in the package. Incomplete GUI/provider/platform checks stay visible in [STATUS.md](STATUS.md); a deferred card is unresolved. See [QUALIFICATION.md](QUALIFICATION.md) for candidate gates.

## Copyable single-card assignment

```text
Repository: D:\dev\Omera. Assigned task: <one ID from W01>.
Read D:\dev\Omera-Review\plans\AGENT_RUNBOOK.md and
D:\dev\Omera-Review\plans\tasks\<ID>.md.
Revalidate current HEAD and integrated task prerequisites.
Implement only this issue and its necessary tests/docs within its owner boundary.
Do not execute the whole package or follow references into new assignments.
Return a concrete handoff with exact commit, acceptance outcomes,
verification results, remaining dependencies and not-run limits.
```

[Coordinator overview](README.md) | [Task index](TASK_INDEX.md) | [Plan review](PLAN_REVIEW.md)
