# W03 Plan: Migration, restore and cleanup validation

**Coordinator:** Lead persistence/security engineer; UI engineer for source-choice consumer  
**Original phase:** 1 | **Cards:** 6

**Outcome:** Source-preserving migration and staged restore use durable, artifact-validated outcomes.

**Original conservative package predecessors:** [W01](W01_PLAN.md), [W02](W02_PLAN.md)

This is a package dispatch/index plan, not one Agent's complete assignment. Individual card prerequisites govern dispatch; starting an independent card before an aggregate package gate requires recorded scope/ownership reasoning. Package completion and release qualification retain their broader gates.

## Dispatch order and coordination

R04-source -> R04 -> R05; then cleanup eligibility and source-choice UI. R01 restore is an independent safety lane after baseline/ownership checks. It shares commands.rs/db.rs with other lead work. Source-choice UI additionally requires the verified R28-api contract.

The table is a stable topological order within this package, not an instruction to run every row in one session. External prerequisites remain explicit.

| Task card | Priority | Profile | Task predecessors |
| --- | --- | --- | --- |
| [R04-source](tasks/R04-source.md) / #269 - Legacy migration reads configuration through a source-mutating loader | P1 | V1 | Baseline/ownership only |
| [R01](tasks/R01.md) / #98 - Cloud restore bypasses the safe database restore lifecycle | P1 | V3 | Baseline/ownership only |
| [R04](tasks/R04.md) / #234 - Legacy configuration migration records success after persistence failure | P1 | V3 | [R04-source](tasks/R04-source.md) |
| [R05](tasks/R05.md) / #235 - Empty destination creation masks pending or failed legacy migration | P1 | V3 | [R04](tasks/R04.md) |
| [R04-cleanup](tasks/R04-cleanup.md) / #270 - Legacy cleanup eligibility lacks configuration and model destination validation | P1 | V3 | [R04](tasks/R04.md), [R05](tasks/R05.md) |
| [R05-ui](tasks/R05-ui.md) / #271 - Provide a legacy source-selection and retry workflow in the application | P2 | V3 | [R04](tasks/R04.md), [R05](tasks/R05.md), [R28-api](tasks/R28-api.md) |

## External task prerequisites

The original aggregate package map is not a complete execution graph. Check these concrete cross-package prerequisites even when a package is absent from the original predecessor list.

| Assigned card | External prerequisite | Owning package |
| --- | --- | --- |
| [R05-ui](tasks/R05-ui.md) | [R28-api](tasks/R28-api.md) | [W01](W01_PLAN.md) |

## Boundaries and exit

Use each card's exact owner and [runbook](AGENT_RUNBOOK.md). Check actual overlapping files and lead-owned contracts before implementation; consult only relevant rows in [CONTRACTS.md](CONTRACTS.md). Preserve all issue-specific acceptance rather than accepting the package by aggregate test success.

Exit requires integrated-candidate acceptance and evidence for every card in the package. Incomplete GUI/provider/platform checks stay visible in [STATUS.md](STATUS.md); a deferred card is unresolved. See [QUALIFICATION.md](QUALIFICATION.md) for candidate gates.

## Copyable single-card assignment

```text
Repository: D:\dev\Omera. Assigned task: <one ID from W03>.
Read D:\dev\Omera-Review\plans\AGENT_RUNBOOK.md and
D:\dev\Omera-Review\plans\tasks\<ID>.md.
Revalidate current HEAD and integrated task prerequisites.
Implement only this issue and its necessary tests/docs within its owner boundary.
Do not execute the whole package or follow references into new assignments.
Return a concrete handoff with exact commit, acceptance outcomes,
verification results, remaining dependencies and not-run limits.
```

[Coordinator overview](README.md) | [Task index](TASK_INDEX.md) | [Plan review](PLAN_REVIEW.md)
