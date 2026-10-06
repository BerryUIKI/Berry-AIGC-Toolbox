# R07-sidecars / #273: Managed import copies prompt-bearing sidecars despite stripping policies

**Assignment:** one issue only. **Package:** [W05](../W05_PLAN.md).  
**Proposed priority:** P1 | **Owner:** Metadata/import engineer; lead review | **Verification:** V1.  
**Tracking:** [#273](https://github.com/BerryUIKI/Omera/issues/273) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R06](R06.md), [R07](R07.md)

**Downstream consumers:** [R23](R23.md) Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Every published sidecar obeys the selected stripping policy.

## Recommended implementation

1. Apply the chosen strip policy before copying/importing any .txt/.json sidecar, including transformed managed import.
2. Ensure filtered SQLite DTOs do not leave unfiltered generation data in destination files. Use the common publisher's sidecar collision/partial-result contract.

## Targeted validation

- [ ] Test prompt-bearing text and JSON under every strip/preserve policy through the actual import service.
- [ ] Inject sidecar publication failures and collisions and verify no privacy leak or pre-existing sidecar overwrite.

## Issue acceptance (preserved verbatim)

- [ ] Filter or suppress sidecars according to policy before publication.
- [ ] Verify that re-import cannot recover stripped fields from copied sidecars.
- [ ] Test transformed import and image/sidecar publication failures.

## Evidence and source pointers

**Code-confirmed.** The transformed managed-import path copies source .txt/.json sidecars without applying the selected metadata stripping policy. Filtering the database DTO does not remove private generation data from these exported/copied files.

1. Create a synthetic prompt-bearing image with matching text and JSON sidecars.
2. Import through the transformed path with StripAi or StripAll.
3. Inspect destination sidecar bytes and re-extract metadata from the destination.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Managed import copies prompt-bearing sidecars despite stripping policies. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src-tauri/src/commands.rs](D:/dev/Omera/src-tauri/src/commands.rs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
