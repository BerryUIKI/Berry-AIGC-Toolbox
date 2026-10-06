# R14-webdav / #275: WebDAV delta sync ignores the selected checksum strategy

**Assignment:** one issue only. **Package:** [W07](../W07_PLAN.md).  
**Proposed priority:** P1 | **Owner:** Sync engineer; lead contract review | **Verification:** V1.  
**Tracking:** [#275](https://github.com/BerryUIKI/Omera/issues/275) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R14](R14.md)

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

The selected strategy is honored, and uncertain content identity is not reported as verified.

## Recommended implementation

1. Honor the selected comparison strategy in WebDAV and preserve a distinction between verified equality and missing/unsupported checksum evidence.
2. Use provider-appropriate metadata or content verification; never fall back to unconditional successful size equality in checksum mode.

## Targeted validation

- [ ] Exercise equal-size different-content objects under each strategy with protocol fixtures.
- [ ] Qualify an isolated live WebDAV provider, including absent/weak ETags, server errors and supported checksum evidence.

## Issue acceptance (preserved verbatim)

- [ ] Implement provider-supported verification for checksum mode or report the unsupported capability.
- [ ] Treat absent/insufficient evidence as unknown and handle it explicitly.
- [ ] Test same-length mismatches using a local protocol fixture.

## Evidence and source pointers

**Code-confirmed; live WebDAV not exercised.** WebDAV delta detection compares only length regardless of the selected checksum strategy, ignoring ETag/checksum evidence. Checksum mode can therefore skip a different equal-length object.

1. Use an isolated WebDAV fixture with different equal-length local/remote content.
2. Select checksum mode and inspect the delta decision.
3. Repeat with changed ETag and unavailable checksum evidence.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: WebDAV delta sync ignores the selected checksum strategy. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src-tauri/src/cloud_sync.rs](D:/dev/Omera/src-tauri/src/cloud_sync.rs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
