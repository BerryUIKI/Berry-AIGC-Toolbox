# W05 Plan: Metadata policies and media round trips

**Coordinator:** Metadata/media engineer with lead persistence/privacy review  
**Original phase:** 2 | **Cards:** 10

**Outcome:** Preservation/stripping policies hold across bytes, sidecars and records; supported formats round-trip.

**Original conservative package predecessors:** [W01](W01_PLAN.md), [W04](W04_PLAN.md)

This is a package dispatch/index plan, not one Agent's complete assignment. Individual card prerequisites govern dispatch; starting an independent card before an aggregate package gate requires recorded scope/ownership reasoning. Package completion and release qualification retain their broader gates.

## Dispatch order and coordination

Metadata policy must be agreed across bytes, sidecars and SQLite. R06/R07 follow R33; their consumers follow individual cards. R36 classification has no task predecessor and can be isolated. AVIF decoder and discovery are distinct acceptances. Do not sweep all ten issues into one metadata rewrite.

The table is a stable topological order within this package, not an instruction to run every row in one session. External prerequisites remain explicit.

| Task card | Priority | Profile | Task predecessors |
| --- | --- | --- | --- |
| [R06](tasks/R06.md) / #236 - Export privacy modes leak the prompt into text sidecars | P1 | V1 | [R33](tasks/R33.md) |
| [R07](tasks/R07.md) / #237 - KeepSupported transformation drops embedded generation metadata | P1 | V1 | [R33](tasks/R33.md) |
| [R36](tasks/R36.md) / #296 - Negative prompt exclusions trigger false-positive NSFW classification | P2 | V1 | Baseline/ownership only |
| [R07-export](tasks/R07-export.md) / #272 - KeepAll export strips embedded metadata during re-encoding | P1 | V1 | [R07](tasks/R07.md) |
| [R07-sidecars](tasks/R07-sidecars.md) / #273 - Managed import copies prompt-bearing sidecars despite stripping policies | P1 | V1 | [R06](tasks/R06.md), [R07](tasks/R07.md) |
| [R09-privacy](tasks/R09-privacy.md) / #274 - StripAll batch transformation retains the original private prompt in SQLite | P1 | V1 | [R07](tasks/R07.md), [R08](tasks/R08.md) |
| [R09](tasks/R09.md) / #239 - Batch transformation leaves database dimensions at their original values | P2 | V1 | [R07](tasks/R07.md), [R08](tasks/R08.md) |
| [R10](tasks/R10.md) / #240 - The app can create AVIF files that its thumbnail decoder cannot read | P2 | V1 | [R07](tasks/R07.md) |
| [R23](tasks/R23.md) / #254 - Managed import bypasses processing for advanced or metadata-only transform settings | P1 | V3 | [R33](tasks/R33.md), [R07-sidecars](tasks/R07-sidecars.md) |
| [R10-discovery](tasks/R10-discovery.md) / #267 - Folder scanning ignores AVIF derivatives created by the application | P2 | V1 | [R10](tasks/R10.md) |

## External task prerequisites

The original aggregate package map is not a complete execution graph. Check these concrete cross-package prerequisites even when a package is absent from the original predecessor list.

| Assigned card | External prerequisite | Owning package |
| --- | --- | --- |
| [R06](tasks/R06.md) | [R33](tasks/R33.md) | [W04](W04_PLAN.md) |
| [R07](tasks/R07.md) | [R33](tasks/R33.md) | [W04](W04_PLAN.md) |
| [R09-privacy](tasks/R09-privacy.md) | [R08](tasks/R08.md) | [W04](W04_PLAN.md) |
| [R23](tasks/R23.md) | [R33](tasks/R33.md) | [W04](W04_PLAN.md) |
| [R09](tasks/R09.md) | [R08](tasks/R08.md) | [W04](W04_PLAN.md) |

## Boundaries and exit

Use each card's exact owner and [runbook](AGENT_RUNBOOK.md). Check actual overlapping files and lead-owned contracts before implementation; consult only relevant rows in [CONTRACTS.md](CONTRACTS.md). Preserve all issue-specific acceptance rather than accepting the package by aggregate test success.

Exit requires integrated-candidate acceptance and evidence for every card in the package. Incomplete GUI/provider/platform checks stay visible in [STATUS.md](STATUS.md); a deferred card is unresolved. See [QUALIFICATION.md](QUALIFICATION.md) for candidate gates.

## Copyable single-card assignment

```text
Repository: D:\dev\Omera. Assigned task: <one ID from W05>.
Read D:\dev\Omera-Review\plans\AGENT_RUNBOOK.md and
D:\dev\Omera-Review\plans\tasks\<ID>.md.
Revalidate current HEAD and integrated task prerequisites.
Implement only this issue and its necessary tests/docs within its owner boundary.
Do not execute the whole package or follow references into new assignments.
Return a concrete handoff with exact commit, acceptance outcomes,
verification results, remaining dependencies and not-run limits.
```

[Coordinator overview](README.md) | [Task index](TASK_INDEX.md) | [Plan review](PLAN_REVIEW.md)
