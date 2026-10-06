# R20-denominator / #280: Prompt statistics counts all indexed files as analyzed

**Assignment:** one issue only. **Package:** [W11](../W11_PLAN.md).  
**Proposed priority:** P2 | **Owner:** Statistics engineer; lead review for SQL | **Verification:** V3.  
**Tracking:** [#280](https://github.com/BerryUIKI/Omera/issues/280) | **Evidence at cutoff:** Source trace.  
**Execution status:** unverified by this split; consult the coordinator's evidence-backed status.

Read [AGENT_RUNBOOK.md](../AGENT_RUNBOOK.md) with this card. Scope comes from the current human assignment; references to other tasks do not assign them.

**Task prerequisites:** [R20](R20.md)

**Downstream consumers:** None recorded. Do not implement them in this assignment.

## Current-candidate check

Evidence cutoff: 2026-10-05; Omera 0.4.3, schema 15, `dev` at `bdac8fbcdbac5535075e091453b77c3afe518ba5`. Record actual HEAD/dirty state, confirm ownership and integrated prerequisites, and reproduce the narrow current production path before editing. The historical report is not current completion status.

## Expected behavior

Statistics clearly define and count the population actually analyzed.

## Recommended implementation

1. Define the analyzed population explicitly and calculate its denominator from records that meet that analysis predicate.
2. Keep totals, percentages and labels aligned with the actual scope, including missing/unparsed metadata.

## Targeted validation

- [ ] Mix analyzed and non-analyzed records and require displayed counts/percentages to match the intended population.
- [ ] Verify empty/all-missing/filtered cases and query bounds without counting every indexed file as analyzed.

## Issue acceptance (preserved verbatim)

- [ ] Query or return the prompt-analysis population explicitly.
- [ ] Use the same population for count labels and derived percentages.
- [ ] Test mixed metadata availability and zero analyzed files.

## Evidence and source pointers

**Code-confirmed.** total_analyzed is assigned the database's total file_count, including files without analyzed prompt metadata. The displayed analyzed count/denominator does not represent the population used for prompt statistics.

1. Create an isolated library with prompt-bearing and metadata-free images.
2. Open prompt statistics.
3. Compare the analyzed count and displayed percentages with the actual included population.

Code-confirmed scenarios above are validation instructions unless the summary explicitly states that the scenario was executed. No destructive real-library or live remote-provider test is implied.

This issue covers one topic: Prompt statistics counts all indexed files as analyzed. Implementation PRs target dev. Preserve source data and follow the lead-owned persistence/security/cleanup boundaries in [ENGINEERING_HANDOFF.md](https://github.com/BerryUIKI/Omera/blob/bdac8fbcdbac5535075e091453b77c3afe518ba5/docs/ENGINEERING_HANDOFF.md).

**Starting touchpoints:** [src/components/PromptStatsModal.vue](D:/dev/Omera/src/components/PromptStatsModal.vue), [docs/ENGINEERING_HANDOFF.md](D:/dev/Omera/docs/ENGINEERING_HANDOFF.md). Historical source line numbers above belong to the cutoff commit; verify current code.

## Delivery

Complete the relevant verification profile and [single-task handoff](../HANDOFF_TEMPLATE.md). Preserve source data and record failed/not-run cases. Update [STATUS.md](../STATUS.md) with evidence; mark verified only after integration and actual issue acceptance. Stop at this assigned issue.
