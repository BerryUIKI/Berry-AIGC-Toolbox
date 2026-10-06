# Advisory contract register

These directions come from the original plan, not from verified implemented IPC. At dispatch, the named lead checks the exact candidate and records the approved design, DTO/version and integration commit in the task handoff. Read only the relevant row. This split approves no new persistence, cleanup or release design.

| Contract | Recommended invariant | Main consumers |
| --- | --- | --- |
| File publication | Reusable no-clobber service; managed destination/type and content identity validation; durable per-item outcomes; explicit interruption/recovery across filesystem and SQLite. | W04/W05; pipeline/watcher callers. |
| Metadata policy | Normalize once across bytes, sidecars, SQLite and showcase; explicit field/container support; unsupported preservation/strip guarantees rejected or clearly exposed. | R06/R07 families, R09-privacy, R23. |
| Migration receipts | Read-only source discovery; each destination artifact validated; pending/failed/skipped distinct; cleanup separately decided and revalidated. | R04 family, R05, R05-ui. |
| Restore | Consistent rollback and validated staging; quiesce submissions/connections; activate before opening database/workers. | R01, R15, release recovery. |
| Query selection | Prefer query descriptor, exclusions/counts and stable generation; bounded jobs. Accepted interim route labels loaded-only scope explicitly. | R35 and batch/checkbox consumers. |
| Config revisions | Revision captured at form load; persist first; mirror/apply accepted values; stale edits report conflict. | R17 family, scheduler defaults. |
| Masking | Shared session presentation; Settings startup default; explicit enabling clears temporary reveals; never reclassify source records. | R21/R37; R36 classification is separate. |
| Sync comparison | Stable root namespace and explicit strategy; verified/unknown distinct; missing digest/weak metadata cannot prove equality; legacy remote layout preserved. | R13/R14 families and transport. |

Required recorded decision fields: owner/reviewer, current implemented capability, invariant, compatibility/recovery behavior, any append-only migration/config default, exact IPC DTO if affected, production acceptance, approval/integration evidence and dependent task IDs. A contract's existence must be confirmed in current code; an advisory row cannot unblock a UI consumer by itself.
