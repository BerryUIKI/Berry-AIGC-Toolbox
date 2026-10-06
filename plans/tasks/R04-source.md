# R04-source / #269: Legacy migration reads configuration through a source-mutating loader

**Assignment:** one issue only. **Package:** [W03](../W03_PLAN.md).  
**Proposed priority:** P1 | **Owner:** Lead migration engineer | **Verification:** V1.  
**Tracking:** [#269](https://github.com/BerryUIKI/Omera/issues/269) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** None beyond baseline/ownership checks.

**Downstream consumers:** [R04](R04.md) Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Migration reads and preserves the legacy source exactly; any conversion is written to the destination.

## Recommended implementation

1. Introduce or use a read-only legacy configuration reader for discovery/migration. Do not normalize/save the source during reading.
2. Carry conversion/credential migration into explicit destination-write steps and report those outcomes separately.

## Targeted validation

- [ ] Hash a synthetic legacy configuration before/after discovery and migration; require identical source bytes.
- [ ] Inject malformed/inaccessible source and destination-write errors without modifying or cleaning the source.

## Issue acceptance (preserved verbatim)

- [ ] Provide a non-mutating source reader for migration.
- [ ] Protect migrated credentials in the destination without rewriting the legacy file.
- [ ] Test exact source-byte preservation on success and on failure.

## Evidence and source pointers

**Code-confirmed.** The coordinator reads the legacy file through config_store::load. That loader may rewrite a configuration to protect plaintext credentials. A migration advertised as preserving its source therefore calls a mutating read API.

1. Create a temporary supported legacy configuration that exercises the loader's credential-protection path; record its bytes/hash.
2. Run configuration migration using an isolated credential store.
3. Compare the legacy source bytes after success and after an injected destination failure.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Legacy migration reads configuration through a source-mutating loader. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src-tauri/src/legacy_migration.rs](D:/dev/Omera/src-tauri/src/legacy_migration.rs), [src-tauri/src/config_store.rs](D:/dev/Omera/src-tauri/src/config_store.rs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
