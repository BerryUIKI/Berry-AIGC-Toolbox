# R07-export / #272: KeepAll export strips embedded metadata during re-encoding

**Assignment:** one issue only. **Package:** [W05](../W05_PLAN.md).  
**Proposed priority:** P1 | **Owner:** Metadata/export engineer; lead review | **Verification:** V1.  
**Tracking:** [#272](https://github.com/BerryUIKI/Omera/issues/272) | **Evidence at cutoff:** Production reproduction.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R07](R07.md)

**Downstream consumers:** [R25-export](R25-export.md) Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

KeepAll preserves supported embedded metadata when export needs re-encoding.

## Recommended implementation

1. Make re-encoded KeepAll export honor the declared preservation contract, sharing metadata handling with transformations where appropriate.
2. Preserve the already passing no-resize byte-exact path; explicitly reject or describe unsupported metadata/container combinations.

## Targeted validation

- [ ] Require prompt preservation for all 25-equivalent prompt-bearing PNG resize/export fixtures.
- [ ] Validate other supported generation fields and ensure original-format pass-through remains byte-exact.

## Issue acceptance (preserved verbatim)

- [ ] Retain supported generation metadata on re-encoding.
- [ ] Define and communicate format-specific preservation limits.
- [ ] Test re-imported output bytes alongside the byte-exact fast path.

## Evidence and source pointers

**Code-confirmed and reproduced with real image copies.** Export's raster encoder strips all embedded metadata even with KeepAll. All 25 prompt-bearing PNG copies lost their prompts after PNG export with max_edge 512. Original-format, no-resize byte pass-through remained exact for all 25.

1. Use a synthetic PNG with supported embedded generation metadata.
2. Export as PNG with KeepAll and resizing, then extract the output metadata.
3. Compare with Original + KeepAll + no resize.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: KeepAll export strips embedded metadata during re-encoding. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [crates/omera-scan/src/export.rs](D:/dev/Omera/crates/omera-scan/src/export.rs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
