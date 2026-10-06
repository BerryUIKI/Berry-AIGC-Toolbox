# Omera remediation: coordinator overview

**Split date / evidence cutoff:** October 5, 2026. **Scope:** 67 topics, 13 packages; 21 proposed P1 and 46 proposed P2. **Status:** planning only; no application fixes, issue closures, test reruns or releases.

The original [REMEDIATION_PLAN.md](../REMEDIATION_PLAN.md) remains the complete reference. This folder is the preferred dispatch entry. Keep the full plan/reports out of ordinary Agent assignments: give one Agent the [runbook](AGENT_RUNBOOK.md), one task card and a concise prerequisite handoff. Read necessary repository instructions and current code on demand.

## Assessment and operating model

The original plan has strong safety and acceptance coverage but mixes coordinator and implementation scope in more than 3,000 lines. Splitting only into 13 documents would still combine ten issues in W05 and nine in W07. Small task cards retain all original issue acceptance and dependencies; package plans supply order/ownership without duplicating all tasks. [PLAN_REVIEW.md](PLAN_REVIEW.md) explains the proposed scheduling adjustments and their limits.

1. Coordinator records the exact `dev` candidate and dirty state, selects a ready card and confirms the appropriate owner/shared-file slot.

2. Start a fresh Gemini/Agent context with the runbook, that card and a short handoff of integrated prerequisites. References are lookup material, not additional tasks.

3. Agent reproduces and implements one issue, runs relevant checks and supplies an evidence-backed [handoff](HANDOFF_TEMPLATE.md).

4. Coordinator reviews/integrates and verifies acceptance on the integrated candidate, then updates [STATUS.md](STATUS.md) and dispatches the next ready card. Preserve unsuccessful/not-run qualification explicitly.

## Package map

| Plan | Phase | Cards / proposed P1 | Owner | Original aggregate predecessors |
| --- | --- | --- | --- | --- |
| [W01: Baseline, regression infrastructure and engineering truth](W01_PLAN.md) | 0 | 8 / 0 | QA engineer with lead review for contract documents | None |
| [W02: Persistent record identity](W02_PLAN.md) | 1 | 1 / 1 | Lead storage engineer | W01 |
| [W03: Migration, restore and cleanup validation](W03_PLAN.md) | 1 | 6 / 5 | Lead persistence/security engineer; UI engineer for source-choice consumer | W01, W02 |
| [W04: Shared file publication and source disposition](W04_PLAN.md) | 1 | 8 / 4 | Lead filesystem engineer | W01, W02 |
| [W05: Metadata policies and media round trips](W05_PLAN.md) | 2 | 10 / 6 | Metadata/media engineer with lead persistence/privacy review | W01, W04 |
| [W06: Reliable discovery and folder ingestion](W06_PLAN.md) | 2 | 3 / 1 | Scan engineer; UI engineer for onboarding; lead reconciliation review | W01, W04 |
| [W07: Synchronization semantics and bounded jobs](W07_PLAN.md) | 2 | 9 / 4 | Lead sync/concurrency engineer | W01, W02, W04 |
| [W08: Authoritative configuration transactions](W08_PLAN.md) | 2 | 3 / 0 | Configuration/UI engineer with lead concurrency review | W01 |
| [W09: History serialization and mutation refresh](W09_PLAN.md) | 2 | 3 / 0 | Frontend state engineer | W01 |
| [W10: Gallery semantics and masking controls](W10_PLAN.md) | 3 | 5 / 0 | Gallery/UI engineer; lead review of query-selection/storage contracts | W01, W05, W06, W09 |
| [W11: Statistics, accessibility, locales and tag discovery](W11_PLAN.md) | 3 | 7 / 0 | UI/accessibility engineer; lead review for new bounded statistics queries | W01, W02 |
| [W12: Optional automation and accurate product capability](W12_PLAN.md) | 3 | 3 / 0 | Backup engineer and documentation engineer; lead scheduler/recovery review | W01, W03, W07 |
| [W13: Signed update delivery and candidate qualification](W13_PLAN.md) | 4 | 1 / 0 | Lead release engineer with QA | W01, W02, W03, W04, W05, W06, W07, W08, W09, W10, W11, W12 |

The inherited package predecessors are provenance, not the complete dispatch graph. W07 also has a concrete W05 prerequisite; W11 has W08/W09/W10 prerequisites; W12 has a W08 prerequisite. Every package plan now lists its external task edges, and [TASK_INDEX.md](TASK_INDEX.md) retains the full task graph.

## First dispatches

- Establish current baseline and run the isolated [R26-format](tasks/R26-format.md) correction if still needed. Add ready frontend/IPC guards and correct stale engineering references as focused cards; they do not collectively block independent reproduction.

- Next ready safety lanes: [R11](tasks/R11.md) row identity after formatting; [R01](tasks/R01.md) restore; [R04-source](tasks/R04-source.md) source preservation; [R12](tasks/R12.md) partial scans; [R14](tasks/R14.md) comparison semantics. These lanes still need named lead ownership and actual shared-file coordination.

- Move [R28](tasks/R28.md) and [R28-remote-db](tasks/R28-remote-db.md) truthful capability documentation into the first wave. Keep R15 backup scheduling behind its recovery/config prerequisites.

- Progress through ready task dependencies rather than waiting for unrelated package completion. Recorded task/package edges are preserved; refactor-before-P1 and presentation/classification edges are advisory review topics, not automatic bypasses.

- Prepare signing infrastructure early where contracts permit; defer release sign-off until the exact candidate meets [qualification gates](QUALIFICATION.md). Deferred features remain unresolved in review accounting.

## Gemini prompt

```text
Work in D:\dev\Omera. Complete only task R26-format.
Read D:\dev\Omera-Review\plans\AGENT_RUNBOOK.md and
D:\dev\Omera-Review\plans\tasks\R26-format.md.
Follow repository AGENTS.md and relevant ownership rules.
Check current HEAD/dirty state and reproduce before editing.
Preserve unrelated changes. Do not execute other cards or the whole plan.
Return exact commit/diff, acceptance and verification outcomes,
not-run cases and remaining dependencies using HANDOFF_TEMPLATE.md.
```

Replace the ID when dispatching another card. For a lead-owned task, confirm its lead owner before implementation; sending the prompt to a general UI Agent does not confer that ownership. Use a new context for each issue; reuse prior concise handoff evidence when the next task depends on it.

## Maintenance

Task/source truth: `evidence/remediation_tasks.json` and frozen `evidence/review_snapshot.json`. The full reference and frozen evidence stay unchanged. Edit dispatch explanations in `scripts/generate_agent_plans.py`; regenerate with `python scripts/generate_agent_plans.py`, then run `python scripts/generate_agent_plans.py --check` from `D:\dev\Omera-Review`. Generated cards/indexes are replaced; `STATUS.md` is created once and preserved thereafter. Keep implementation logs outside generated cards.

Use [TASK_INDEX.md](TASK_INDEX.md) for all issue/prerequisite mappings; [CONTRACTS.md](CONTRACTS.md) for advisory decisions. The completed performance manuscript in a separate repository remains outside this task.
