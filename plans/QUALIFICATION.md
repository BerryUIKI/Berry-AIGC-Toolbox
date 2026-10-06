# Coordinator qualification and exit gates

Preserved from sections 7-8 of the original plan. This is candidate/package qualification reference; individual Agents use their card and runbook. No cases below were rerun by splitting documents. Task verification and release sign-off are separate outcomes.

## 7. Safety and regression matrix

| Boundary | Minimum injected/interaction cases | Expected invariant |
| --- | --- | --- |
| File publication | Identical content, equal-length different content, different-length collision, sidecar collision, publish/DB failure, interruption/retry | Existing assets/source identity survive; receipts name actual outcomes; no clobber or false deduplication. |
| Source disposition | DB commit failure, archive/trash failure, changed source path identity, missing destination, deferred expiry | Original remains recoverable and cleanup never acts without current validated proof. |
| Migration | Nonzero legacy revision, failed config/credential save, mutated source risk, empty destination, multiple candidates, missing config/model destination | No false success/eligibility; source remains unchanged; retry/source-choice reachable. |
| Restore | Active WAL/mmap, failed consistent rollback, invalid schema/integrity/FKs, restart/apply interruption | Stage and apply safely before connections; rollback/retry preserves a recoverable state. |
| Privacy/preservation | Supported/unsupported policy/container, re-encoding/pass-through, .txt/.json, SQLite and showcase, advanced-only import | Policy matches every representation; unsupported semantics are explicit; curation stays attached. |
| Media/scanner | AVIF generated/discovered/decoded, malformed input, partial/inaccessible walks, initial folder and stable watcher events | Exposed formats round-trip; uncertain absence does not delete records; existing files become visible. |
| Config/history | Backend rejection, stale revision, rapid/reentrant undo/redo, command failure, mirror restart | Only accepted state becomes active; failed transitions stay retryable and ordering deterministic. |
| Gallery/state | Initial/appended/full query selection, filters, stacks, stale detail responses, masking/reveal, narrow/wide/reduced motion | Accurate scope/counts and consistent presentation with bounded work and keyboard access. |
| Cloud | Same-name roots, same-size different data, absent/wrong digests, weak metadata, low-bandwidth wait, stalled request | No unverified skip/collision; bounded memory; cancelable honest outcomes and compatible keys. |
| Release | Correct/wrong trust key, missing/tampered signatures, installer/update interruption, migration after update | Fail closed for untrusted assets; supported candidate preserves data/credentials and recovers. |

Use local/protocol fixtures first, then isolated live providers and real target installers where required. Record skipped/not-run cases explicitly. The prior closed-connection two-row restore and normal gallery rendering do not substitute for active restore, minimum-width, reduced-motion or platform qualification.

## 8. Milestone exit gates

| Gate | Exit requirement |
| --- | --- |
| G0 - reproducible baseline | Current commit/dirty state documented; maintained tests and IPC/format guards run; stale contract/status references corrected without erasing safety ownership. |
| G1 - safe data lifecycle | All P1 publication, identity, policy, migration, cleanup and recovery tasks pass their actual production/fault cases; sources/destinations and receipts remain consistent. |
| G2 - bounded reliable jobs | Sync strategies/namespaces, cancellation and database lock scopes are verified; watchers/scheduler have start/stop/retry ownership and no overlapping jobs. |
| G3 - coherent product | Onboarding, selection, stacks, masking, config/history, statistics, keyboard/locales and tag/table proposals meet issue criteria in native/production-component tests. |
| G4 - qualified delivery | Required candidate checks and supported-platform/provider/update matrix pass; README/release/engineering claims match verified capability; lead records remaining limitations. |
| G5 - complete review disposition | Every one of the 67 IDs has a linked implemented/verified disposition; waived/deferred/administratively closed topics remain explicitly unresolved unless the project consciously changes scope. |

No date/effort commitment is made until owners validate current source, fixtures, dependencies and environment. Do not claim that all issues are solved while any has only a proposal, code presence, a non-executed acceptance instruction or an unrelated passing path.

[Overview](README.md) | [Execution status](STATUS.md)
