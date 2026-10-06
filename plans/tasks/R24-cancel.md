# R24-cancel / #283: Cloud sync cancellation does not interrupt limiter waits or active transfers

**Assignment:** one issue only. **Package:** [W07](../W07_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Lead sync/concurrency engineer | **Verification:** V1.  
**Tracking:** [#283](https://github.com/BerryUIKI/Omera/issues/283) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R24](R24.md)

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Cancellation interrupts waiting and active transfer work promptly.

## Recommended implementation

1. Replace whole-file limiter sleeps under the shared lock with interruptible waits and release unrelated scheduling state promptly.
2. Propagate cancellation into active requests with explicit timeout/abort semantics and truthful per-item canceled/partial outcomes.

## Targeted validation

- [ ] Cancel while waiting on low bandwidth allowance, during a stalled transfer and between chunks.
- [ ] Require bounded cancellation completion under controlled fixtures and qualify actual provider request cancellation without false success counts.

## Issue acceptance (preserved verbatim)

- [ ] Make limiter waits cancellable without holding the shared lock during sleep.
- [ ] Use bounded transfer steps/timeouts with cancellation checks.
- [ ] Test cancel latency during throttling, hashing and stalled transport.

## Evidence and source pointers

**Code-confirmed.** The limiter sleeps for a whole file's allowance while holding its shared lock. Cancellation is checked between work steps, but not during the limiter sleep or transfer; a low bandwidth limit or stalled transfer can delay cancellation substantially.

1. Use a local slow transport and a synthetic file with a low configured bandwidth limit.
2. Cancel during the limiter wait and during upload.
3. Measure time until the job stops and verify its receipt.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Cloud sync cancellation does not interrupt limiter waits or active transfers. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src-tauri/src/cloud_sync.rs](D:/dev/Omera/src-tauri/src/cloud_sync.rs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
