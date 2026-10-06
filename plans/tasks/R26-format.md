# R26-format / #294: Existing Rust formatting drift fails CI on every platform

**Assignment:** one issue only. **Package:** [W01](../W01_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Rust engineer | **Verification:** V0.  
**Tracking:** [#294](https://github.com/BerryUIKI/Omera/issues/294) | **Evidence at cutoff:** Automated check.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** None beyond baseline/ownership checks.

**Downstream consumers:** [R11](R11.md) Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

The integration branch passes the repository's formatting gate.

## Recommended implementation

1. Apply rustfmt to the specific pre-existing command formatting differences and preserve all unrelated local changes.
2. Keep the formatting PR isolated from behavior changes so the corrected baseline is reviewable.

## Targeted validation

- [ ] Run cargo fmt --check locally.
- [ ] Verify the affected platform CI jobs proceed past formatting; do not treat that as platform runtime acceptance.

## Issue acceptance (preserved verbatim)

- [ ] Apply the required formatting changes without runtime behavior changes.
- [ ] Verify cargo fmt --check succeeds.
- [ ] Confirm the formatting step passes on the supported CI matrix.

## Evidence and source pointers

**Reproduced locally and observed in the template PR CI.** cargo fmt --check fails in commands.rs at the reviewed transform-query formatting sites. All three platform checks on PR #242 failed at this pre-existing formatting check; the PR changed only issue templates and CONTRIBUTING.md.

1. Run cargo fmt --check from dev.
2. Inspect the formatting-only diffs around percentage expression, trailing comma and iterator formatting.
3. Compare the three platform logs on PR #242.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Existing Rust formatting drift fails CI on every platform. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

Related evidence: [PR #242](https://github.com/BerryUIKI/Omera/pull/242).

**Starting touchpoints:** [src-tauri/src/commands.rs](D:/dev/Omera/src-tauri/src/commands.rs), [.github/workflows/ci.yml](D:/dev/Omera/.github/workflows/ci.yml), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
