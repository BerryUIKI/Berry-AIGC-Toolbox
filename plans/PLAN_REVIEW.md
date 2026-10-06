# Review of the original remediation plan

Reviewed October 5, 2026. This is a structural, dependency and dispatch review of the local plan and its frozen JSON sources, with repository ownership/roadmap context. It does not reproduce the 67 application findings, query current GitHub issue state or qualify a release.

## Assessment

The plan is a sound review register and coordinator reference. It preserves issue identity, owners, data-safety boundaries, actual production-path/failure verification, honest unexecuted cases and explicit delivery gates. Its issue-level graph is acyclic and its 67 topics are covered once. Those properties should be retained.

It is a poor single Agent assignment: more than 3,000 lines mix execution rules, scheduling, proposed contracts, 13 packages, 67 issues and release acceptance. Blank-line compression would shorten display without reducing the number of decisions or preventing scope drift. W05 has ten issues and W07 nine; copying a whole package still assigns too much shared state to one context.

## Changes to dispatch, with original acceptance preserved

| Concern | Evidence/example | Revised handling |
| --- | --- | --- |
| Execution scope is too broad | One file combines all 67 topics and several owners. | One assignment = runbook + one task card. Package plans guide the coordinator. A fresh context receives the prior handoff, not the full transcript. |
| Package gates overstate start dependencies | W03 follows W01/W02, but R01 has no task predecessor; W06 also lists W04 while R12 has none. | Task prerequisites govern card dispatch; original package predecessors stay visible as conservative aggregate gates. Record the rationale when starting independent cards early. |
| Package graph misses concrete cross-package prerequisites | W07's R25-export needs W05's R06/R07-export; W11's R22-labels needs W10's R35 and R29 needs W08/R17 plus W09/R18; W12/R15 also needs W08/R17. These packages are absent from the corresponding original predecessor ancestry. | Generate an external-task dependency table in every package plan. The original package map is retained for provenance and cannot substitute for the full task graph. |
| Early claim correction is buried | W12 includes R28/R28-remote-db even though phase 0 asks for early correction. | Schedule those two documentation cards in the first wave; leave R15 behind safe recovery/config. |
| P1 repairs depend on large P2 extraction | R02 follows R33-pipeline, which follows R33. | Lead should review a bounded no-clobber repair versus extraction risk. Any alternate ordering needs an explicit dependency change and unchanged acceptance; this split does not grant a bypass. |
| Design suggestions may become accidental APIs | Query-selection descriptors, metadata matrices and receipts are recommendations. | Separate contract register with explicit advisory status and exact-candidate implementation/DTO checks. Do not turn a suggestion into a consumer assumption. |
| Some edges may be coordination gates | R37 masking depends on R36 classification although the toolbar must never reclassify files; R22-labels depends on R35 selection. | Preserve both edges and request lead review of start versus completion meaning; independent UI investigation may precede integration. |
| Preparation differs from release sign-off | W13 lists all packages, but signing setup can start earlier. | Prepare signing after required contracts; qualify the exact candidate only after safety and relevant product gates. |
| Historical evidence can be mistaken for current status | Runtime baseline is bdac8fb; later a4ac24f is documentation/template work. | Keep cutoff immutable; initialize execution status as unverified; revalidate actual HEAD and dirty state per assignment. |
| Split docs can drift or erase progress | The original Markdown is generated from frozen JSON. | Generate split task cards from the same JSON; check IDs, acceptance, dependencies, links and size. Keep STATUS.md outside generated replacement. |

## Scheduling judgement

Keep the 21 proposed P1 topics ahead of broad-release sign-off after current-candidate revalidation. "P2" is not one homogeneous queue: it includes refactoring, functional defects, automation, release signing and product proposals. Order by dependency and risk, and distinguish review completion from a narrower release with explicit unresolved/deferred scope. Deferral never becomes a verified fix.

Start with a recorded baseline, the isolated formatting correction and the available regression/contract guards. Move row identity, restore, migration source preservation, partial-scan safety and comparison correctness into the next ready safety lanes. Do not wait for every W01/W04 card before investigating independent P1s. Coordinate actual overlapping files before implementing those lanes.

Full GUI/provider/platform acceptance remains essential where required. Record unavailable native/live-provider/installer evidence as pending qualification; do not replace it with source assertions or the document generator's successful checks.

The split retains all frozen task and package edges, all issue acceptance, all implementation/targeted-validation text and the evidence cutoff. Proposed ordering changes above are clearly advisory, not silently edited dependencies.
