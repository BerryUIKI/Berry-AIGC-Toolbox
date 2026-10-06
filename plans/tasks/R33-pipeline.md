# R33-pipeline / #293: Extract pipeline harvesting and cleanup business logic from Tauri command adapters

**Assignment:** one issue only. **Package:** [W04](../W04_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Lead file-service engineer | **Verification:** V3.  
**Tracking:** [#293](https://github.com/BerryUIKI/Omera/issues/293) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R33](R33.md)

**Downstream consumers:** [R02](R02.md), [R02-dedup](R02-dedup.md), [R16](R16.md) Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Pipeline business rules live in reusable services with explicit input/output contracts.

## Recommended implementation

1. Move harvesting and deferred cleanup orchestration into reusable Rust services; keep Tauri commands limited to input validation, short locks and delegation.
2. Preserve cancellation/progress contracts and expose structured per-item outcomes so collision and cleanup safety can be tested without invoking the UI.

## Targeted validation

- [ ] Execute actual production services with synthetic pipeline sources, collision outcomes, receipt failures and deferred cleanup.
- [ ] Verify command DTO compatibility and that cleanup still requires destination/source revalidation.

## Issue acceptance (preserved verbatim)

- [ ] Extract harvesting/cleanup orchestration behind thin adapters.
- [ ] Use collision and cleanup-revalidation issues as production-service acceptance tests.
- [ ] Preserve explicit cleanup decisions and never include user media/external/shared roots in automatic legacy cleanup.

## Evidence and source pointers

**Code-confirmed architectural debt.** Pipeline harvesting and deferred cleanup implement substantial filesystem/publication/receipt policy directly in commands.rs, although repository boundaries require reusable Rust services and thin adapters. This makes fault injection and consistent file-safety validation across jobs difficult.

1. Trace manual pipeline harvest and deferred cleanup in command adapters.
2. Identify publication, cleanup eligibility and per-item receipt decisions that can be tested without Tauri.
3. Define the service boundary before moving behavior.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Extract pipeline harvesting and cleanup business logic from Tauri command adapters. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src-tauri/src/commands.rs](D:/dev/Omera/src-tauri/src/commands.rs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
