# R28-remote-db / #289: README claims shipped MySQL and PostgreSQL storage while runtime is SQLite-only

**Assignment:** one issue only. **Package:** [W12](../W12_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Documentation engineer | **Verification:** V5.  
**Tracking:** [#289](https://github.com/BerryUIKI/Omera/issues/289) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** None beyond baseline/ownership checks.

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Storage documentation accurately identifies the shipped runtime backend.

## Recommended implementation

1. State that the current runtime is SQLite-backed and remote MySQL/PostgreSQL team storage is unavailable/planned.
2. Separate SQL export, traits and connection tests from a functioning application storage backend; do not enable a nonimplemented backend just to match marketing text.

## Targeted validation

- [ ] Compare README/settings and persisted configuration behavior with the actual database runtime.
- [ ] Confirm no user-facing language implies disabled remote choices are a shipped collaboration feature.

## Issue acceptance (preserved verbatim)

- [ ] Describe SQLite as the supported runtime.
- [ ] Identify remote database support as planned/experimental if that is its actual status.
- [ ] Align user setup instructions and translated claims.

## Evidence and source pointers

**Code-confirmed documentation mismatch.** The README advertises full MySQL/PostgreSQL team storage. Runtime uses Mutex<Database> with SQLite, configuration load/save forces SQLite and remote backend choices are disabled in Settings. SQL export/traits do not establish a working remote database runtime.

1. Compare team-storage claims with configuration normalization, AppState database type and disabled backend choices.
2. Attempt to locate an implemented connection lifecycle/query backend for either advertised server.
3. Distinguish SQL export from a selectable running storage backend.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: README claims shipped MySQL and PostgreSQL storage while runtime is SQLite-only. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [README.md](D:/dev/Omera/README.md), [src-tauri/src/commands.rs](D:/dev/Omera/src-tauri/src/commands.rs), [src/components/SettingsModal.vue](D:/dev/Omera/src/components/SettingsModal.vue), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
