# R10-discovery / #267: Folder scanning ignores AVIF derivatives created by the application

**Assignment:** one issue only. **Package:** [W05](../W05_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Scan/media engineer | **Verification:** V1.  
**Tracking:** [#267](https://github.com/BerryUIKI/Omera/issues/267) | **Evidence at cutoff:** Production reproduction.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R10](R10.md)

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

The scanner discovers app-produced AVIF files according to the advertised format support.

## Recommended implementation

1. Include AVIF in discovery only with the approved supported format contract and correct container detection.
2. Keep discovery and decoder acceptance separate so fixing one cannot conceal the other.

## Targeted validation

- [ ] Scan an app-generated AVIF-only directory and require the supported derivative to be found/added.
- [ ] Test mixed PNG/AVIF and malformed input; verify scan and thumbnail outcomes independently.

## Issue acceptance (preserved verbatim)

- [ ] Include AVIF in scanner discovery and use appropriate container detection.
- [ ] Test an AVIF-only directory and mixed PNG/AVIF directories.
- [ ] Keep decoder support tracked in #240 so discovery and rendering can be verified separately.

## Evidence and source pointers

**Reproduced with production services on a copied real image.** transform_file_staged successfully creates an AVIF derivative, but Scanner::scan_folder over the output-only directory reports found=0 and added=0. MEDIA_EXTENSIONS omits avif. This is a discovery defect separate from the AVIF thumbnail decoder issue #240.

1. Create an isolated AVIF-only directory using transform_file_staged on a temporary PNG fixture.
2. Register that directory in a temporary database and run Scanner::scan_folder.
3. Compare the existing AVIF file with the scan's found/added counts.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Folder scanning ignores AVIF derivatives created by the application. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [crates/omera-scan/src/scanner.rs](D:/dev/Omera/crates/omera-scan/src/scanner.rs), [crates/omera-scan/src/transform.rs](D:/dev/Omera/crates/omera-scan/src/transform.rs), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
