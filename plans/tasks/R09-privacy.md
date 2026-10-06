# R09-privacy / #274: StripAll batch transformation retains the original private prompt in SQLite

**Assignment:** one issue only. **Package:** [W05](../W05_PLAN.md).  
**Proposed priority:** P1 | **Owner:** Lead transform/privacy engineer | **Verification:** V1.  
**Tracking:** [#274](https://github.com/BerryUIKI/Omera/issues/274) | **Evidence at cutoff:** Production reproduction.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R07](R07.md), [R08](R08.md)

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

The transformed record's generation metadata matches the selected privacy policy.

## Recommended implementation

1. Build the updated record from validated policy-filtered derivative metadata, removing stripped prompt/AI fields from SQLite as well as image bytes.
2. Preserve user curation fields and explicit manual overrides independently of extracted generation metadata.

## Targeted validation

- [ ] Re-run successful StripAll batch transform and require no original prompt/AI metadata in the updated row, details or exports.
- [ ] Test rollback/failure and keep policies to ensure user albums/tags/ratings remain attached correctly.

## Issue acceptance (preserved verbatim)

- [ ] Persist policy-consistent extracted metadata in the transformation transaction.
- [ ] Remove stripped prompt/AI fields while preserving unrelated user curation.
- [ ] Test database and output-byte privacy independently.

## Evidence and source pointers

**Reproduced.** A successful StripAll batch transformation keeps the original ExtractedMetadata on the updated file row, including its private prompt. The derivative pixels may be clean, but the transformed library record still exposes data the selected policy says to remove.

1. Index a synthetic prompt-bearing image in an isolated managed library.
2. Run batch transformation with StripAll and Keep originals.
3. Read metadata from the transformed file record and compare with the derivative.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: StripAll batch transformation retains the original private prompt in SQLite. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [crates/omera-storage/src/db.rs](D:/dev/Omera/crates/omera-storage/src/db.rs), [crates/omera-scan/src/transform.rs](D:/dev/Omera/crates/omera-scan/src/transform.rs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
