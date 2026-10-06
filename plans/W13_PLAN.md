# W13 Plan: Signed update delivery and candidate qualification

**Coordinator:** Lead release engineer with QA  
**Original phase:** 4 | **Cards:** 1

**Outcome:** The exact release candidate passes signed-update, platform and recovery acceptance without bypassing verification.

**Original conservative package predecessors:** [W01](W01_PLAN.md), [W02](W02_PLAN.md), [W03](W03_PLAN.md), [W04](W04_PLAN.md), [W05](W05_PLAN.md), [W06](W06_PLAN.md), [W07](W07_PLAN.md), [W08](W08_PLAN.md), [W09](W09_PLAN.md), [W10](W10_PLAN.md), [W11](W11_PLAN.md), [W12](W12_PLAN.md)

This is a package dispatch/index plan, not one Agent's complete assignment. Individual card prerequisites govern dispatch; starting an independent card before an aggregate package gate requires recorded scope/ownership reasoning. Package completion and release qualification retain their broader gates.

## Dispatch order and coordination

Signing/configuration preparation may start before the whole review is complete. R27 task predecessors remain R01, R26 and R26-ipc-ci. Candidate release qualification additionally uses every relevant package/safety/product gate; a working signature alone is not release approval.

The table is a stable topological order within this package, not an instruction to run every row in one session. External prerequisites remain explicit.

| Task card | Priority | Profile | Task predecessors |
| --- | --- | --- | --- |
| [R27](tasks/R27.md) / #256 - The checked-in release pipeline does not provision signed automatic updates | P2 | V6 | [R01](tasks/R01.md), [R26](tasks/R26.md), [R26-ipc-ci](tasks/R26-ipc-ci.md) |

## External task prerequisites

The original aggregate package map is not a complete execution graph. Check these concrete cross-package prerequisites even when a package is absent from the original predecessor list.

| Assigned card | External prerequisite | Owning package |
| --- | --- | --- |
| [R27](tasks/R27.md) | [R01](tasks/R01.md) | [W03](W03_PLAN.md) |
| [R27](tasks/R27.md) | [R26](tasks/R26.md) | [W01](W01_PLAN.md) |
| [R27](tasks/R27.md) | [R26-ipc-ci](tasks/R26-ipc-ci.md) | [W01](W01_PLAN.md) |

## Boundaries and exit

Use each card's exact owner and [runbook](AGENT_RUNBOOK.md). Check actual overlapping files and lead-owned contracts before implementation; consult only relevant rows in [CONTRACTS.md](CONTRACTS.md). Preserve all issue-specific acceptance rather than accepting the package by aggregate test success.

Exit requires integrated-candidate acceptance and evidence for every card in the package. Incomplete GUI/provider/platform checks stay visible in [STATUS.md](STATUS.md); a deferred card is unresolved. See [QUALIFICATION.md](QUALIFICATION.md) for candidate gates.

## Copyable single-card assignment

```text
Repository: D:\dev\Omera. Assigned task: <one ID from W13>.
Read D:\dev\Omera-Review\plans\AGENT_RUNBOOK.md and
D:\dev\Omera-Review\plans\tasks\<ID>.md.
Revalidate current HEAD and integrated task prerequisites.
Implement only this issue and its necessary tests/docs within its owner boundary.
Do not execute the whole package or follow references into new assignments.
Return a concrete handoff with exact commit, acceptance outcomes,
verification results, remaining dependencies and not-run limits.
```

[Coordinator overview](README.md) | [Task index](TASK_INDEX.md) | [Plan review](PLAN_REVIEW.md)
