# R02-dedup / #268: Managed import treats equal byte lengths as identical content

**Assignment:** one issue only. **Package:** [W04](../W04_PLAN.md).  
**Proposed priority:** P1 | **Owner:** Lead import/pipeline engineer | **Verification:** V1.  
**Tracking:** [#268](https://github.com/BerryUIKI/Omera/issues/268) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R33](R33.md), [R33-pipeline](R33-pipeline.md)

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Content identity is verified before an existing managed asset is reused.

## Recommended implementation

1. Use length/name only as candidate filters; establish actual content identity through the approved digest or byte comparison before deduplication.
2. Apply the same identity rule to plain managed import and pipeline harvest, with collision receipts linked to the correct file ID.

## Targeted validation

- [ ] Provide different equal-length content with the same basename to both production callers and require separate preserved assets.
- [ ] Verify identical content may deduplicate without copying or modifying user organization and returns the correct record identity.

## Issue acceptance (preserved verbatim)

- [ ] Use a content digest or equivalent verified identity check before deduplication.
- [ ] Preserve distinct equal-length assets under separate names.
- [ ] Test identical content, equal-size different content and concurrent publication.

## Evidence and source pointers

**Code-confirmed.** Plain managed import and pipeline harvest use matching basename plus file length as a deduplication check. Different equal-length content is silently skipped or assigned the existing file ID. The two callers share the same incorrect identity rule.

1. Create two distinct valid images with the same basename and equal byte length in temporary source/destination folders.
2. Run plain managed import, then repeat via manual pipeline harvesting.
3. Compare hashes, imported IDs and receipts.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Managed import treats equal byte lengths as identical content. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src-tauri/src/commands.rs](D:/dev/Omera/src-tauri/src/commands.rs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
