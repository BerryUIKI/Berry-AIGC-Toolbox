# W07 Plan: Synchronization semantics and bounded jobs

**Coordinator:** Lead sync/concurrency engineer  
**Original phase:** 2 | **Cards:** 9

**Outcome:** Stable remote namespaces, verified comparisons, cancelable transfers and short database lock scopes.

**Original conservative package predecessors:** [W01](W01_PLAN.md), [W02](W02_PLAN.md), [W04](W04_PLAN.md)

This is a package dispatch/index plan, not one Agent's complete assignment. Individual card prerequisites govern dispatch; starting an independent card before an aggregate package gate requires recorded scope/ownership reasoning. Package completion and release qualification retain their broader gates.

## Dispatch order and coordination

There are separate remote namespace, comparison/transport, and database lock lanes. R14 and R25-estimate have no task predecessors. R13, R25 and R25-export use the cross-package prerequisites in their cards. Provider memory/cancellation work follows the comparison contract. Serialize changes to cloud_sync.rs and commands.rs as needed.

The table is a stable topological order within this package, not an instruction to run every row in one session. External prerequisites remain explicit.

| Task card | Priority | Profile | Task predecessors |
| --- | --- | --- | --- |
| [R13](tasks/R13.md) / #244 - Cloud object keys collide between library roots with the same basename | P1 | V3 | [R11](tasks/R11.md), [R33](tasks/R33.md) |
| [R14](tasks/R14.md) / #245 - FastFingerprint sync skips same-length content changes | P1 | V1 | Baseline/ownership only |
| [R25](tasks/R25.md) / #202 - Batch transformations retain the shared database mutex during image processing | P2 | V3 | [R08](tasks/R08.md) |
| [R25-estimate](tasks/R25-estimate.md) / #284 - Export estimates retain the shared database mutex during decoding and encoding | P2 | V3 | Baseline/ownership only |
| [R25-export](tasks/R25-export.md) / #285 - Batch export retains the shared database mutex throughout transcoding and output writes | P2 | V3 | [R06](tasks/R06.md), [R07-export](tasks/R07-export.md) |
| [R14-webdav](tasks/R14-webdav.md) / #275 - WebDAV delta sync ignores the selected checksum strategy | P1 | V1 | [R14](tasks/R14.md) |
| [R14-s3](tasks/R14-s3.md) / #276 - S3 checksum mode treats a missing remote digest as a verified size match | P1 | V1 | [R14](tasks/R14.md) |
| [R24](tasks/R24.md) / #255 - Cloud sync allocates whole files without a global memory budget | P2 | V1 | [R14](tasks/R14.md) |
| [R24-cancel](tasks/R24-cancel.md) / #283 - Cloud sync cancellation does not interrupt limiter waits or active transfers | P2 | V1 | [R24](tasks/R24.md) |

## External task prerequisites

The original aggregate package map is not a complete execution graph. Check these concrete cross-package prerequisites even when a package is absent from the original predecessor list.

| Assigned card | External prerequisite | Owning package |
| --- | --- | --- |
| [R13](tasks/R13.md) | [R11](tasks/R11.md) | [W02](W02_PLAN.md) |
| [R13](tasks/R13.md) | [R33](tasks/R33.md) | [W04](W04_PLAN.md) |
| [R25](tasks/R25.md) | [R08](tasks/R08.md) | [W04](W04_PLAN.md) |
| [R25-export](tasks/R25-export.md) | [R06](tasks/R06.md) | [W05](W05_PLAN.md) |
| [R25-export](tasks/R25-export.md) | [R07-export](tasks/R07-export.md) | [W05](W05_PLAN.md) |

## Boundaries and exit

Use each card's exact owner and [runbook](AGENT_RUNBOOK.md). Check actual overlapping files and lead-owned contracts before implementation; consult only relevant rows in [CONTRACTS.md](CONTRACTS.md). Preserve all issue-specific acceptance rather than accepting the package by aggregate test success.

Exit requires integrated-candidate acceptance and evidence for every card in the package. Incomplete GUI/provider/platform checks stay visible in [STATUS.md](STATUS.md); a deferred card is unresolved. See [QUALIFICATION.md](QUALIFICATION.md) for candidate gates.

## Copyable single-card assignment

```text
Repository: D:\dev\Omera. Assigned task: <one ID from W07>.
Read D:\dev\Omera-Review\plans\AGENT_RUNBOOK.md and
D:\dev\Omera-Review\plans\tasks\<ID>.md.
Revalidate current HEAD and integrated task prerequisites.
Implement only this issue and its necessary tests/docs within its owner boundary.
Do not execute the whole package or follow references into new assignments.
Return a concrete handoff with exact commit, acceptance outcomes,
verification results, remaining dependencies and not-run limits.
```

[Coordinator overview](README.md) | [Task index](TASK_INDEX.md) | [Plan review](PLAN_REVIEW.md)
