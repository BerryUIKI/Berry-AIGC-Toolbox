# Omera storage evolution

Status: design direction; optimizations require measurements and append-only migrations.

Current transformed records: the existing files.metadata JSON stores verified published pixel dimensions in the same update as path/container/size/mtime. Raw generator Size parameters remain provenance; metadata.format is null for dimension-only records after stripping or unsupported metadata extraction. No schema version change, new index, or curation identity change is needed.

Ownership: lead maintainer/Codex. General engineers may supply fixtures and benchmark results; authoritative schema, migration, recovery and database-layout changes remain lead-owned.

## Decision

Retain embedded SQLite with WAL as the authoritative local library, renamed to `omera.db`. The current application already relies on relational transactions for files, folders, tags, albums, stacks, and recovery. Renaming or replacing the engine alone does not make those workloads faster. Remote MySQL/PostgreSQL support is not implemented end to end and must not be advertised as available.

Keep original media on disk, large model weights outside the database, and thumbnails in a disposable bounded disk cache. Preserve existing IDs, relationships, metadata, embeddings, and schema version during identity migration. Do not rebuild a user's library from a filesystem scan as a substitute for migration.

## Workload-driven changes

| Workload | Proposed change | Evidence required |
| --- | --- | --- |
| Gallery pages | Small typed projections, stable keyset cursors with ID tie-breaks, indexes matching scope and ordering | EXPLAIN QUERY PLAN and first/deep-page p50/p95 latency |
| Metadata filters | Extract frequently queried fields into typed indexed columns or a one-to-one metadata table; keep full source JSON separately | Lower page/filter latency and IPC size without lost search semantics |
| Full-text search | Evaluate SQLite FTS5 alongside existing behavior | Multilingual tokenization, substring expectations, update cost, index size and correctness |
| Sidebar totals | Aggregate counts in bounded query groups and invalidate by mutation category | Startup query count and refresh latency, with accurate counts |
| Thumbnail bookkeeping | Reuse worker connections, coalesce access/budget writes, bound eviction batches | Decode throughput, lock wait and write amplification |
| Semantic search | Bounded top-k selection and bulk hydration first; evaluate optional vector indexes later | Exact-search baseline, recall@k, memory, latency and packaging cost |
| Background indexing | Short bounded write transactions and explicit busy handling; separate background reads | UI latency while scan/inference/cache jobs run together |

Do not add indexes for every column or introduce a second authoritative database without measurements. Separating cache metadata into another SQLite database may reduce contention, but also loses cross-database foreign-key guarantees and complicates recovery; retain the existing layout until contention measurements justify it. Optional vector indexes must be rebuildable derivatives of versioned authoritative embeddings.

## Schema and migration rules

Library batch transformations take short locks for folder/file reads and the
single-row derivative update. Decode, verification, publication, archive/Trash
and progress callbacks run outside the application database mutex. Transform
jobs remain serialized by a separate job gate to avoid overlapping staging.
Publication uses an atomic compare-and-update against the original row's path,
folder, size and modification time. A changed/deleted row rejects publication
and preserves the source; the derivative is compensated using the existing
failure path. Curation fields are never rewritten from the earlier snapshot.
No schema migration is needed. This is not a general filesystem transaction.

- Applied migrations remain unchanged and ordered. Append new migrations in the storage crate; crate renaming moves the file without rewriting its history.
- Backfill large new structures in bounded resumable batches. Do not expose incomplete indexes as complete search results. Define schema compatibility and recovery before switching readers.
- Use foreign keys, uniqueness constraints, and explicit version/revision fields where required by actual ownership. Verify `integrity_check` and `foreign_key_check` on migrated copies.
- Store migration receipts outside the source database so failure never requires modifying the only good copy. Legacy discovery and cleanup follow [OMERA_MIGRATION.md](OMERA_MIGRATION.md).
- Keep authoritative databases on local storage; media may reside on network shares. A shared multi-user service is a separate future architecture, not a connection-string toggle.

### Cloud root identity foundation (schema v17)

`cloud_sync_roots` maps each registered folder ID to an immutable random UUID
and the basename observed at first allocation. `Database::ensure_cloud_sync_root`
allocates lazily; repeated calls, folder path changes, restart, and SQLite snapshots
retain the mapping. Folder removal cascades the mapping so registering another
folder with a reused numeric ID creates a new UUID. Existing folders, files,
curation, and migrations remain intact; there is no eager library backfill.

Cloud sync now previews and saves a compatibility manifest and requires explicit
user acknowledgement before publishing under the versioned UUID namespace. It
retains legacy remote objects and source media, identifies ambiguous mappings,
and never infers authorization to delete or adopt them. This table is separate from
`storage_roots`, whose workstation mappings are not linked to registered folders.

`omera_scan::cloud_sync` provides the shared planning services:
`build_namespace_manifest`, `collect_sync_plan`, and `save_namespace_manifest`.
They map selected roots to `<prefix>/v2-root-<uuid>`, flag ambiguous legacy
basenames (including unselected registered roots), and reject unsafe paths or
case-insensitive duplicate keys before returning any transfer queue. The plan
requires acknowledgement of the current manifest's SHA-256 ID. Manifests are
atomically saved under `cloud-sync-manifests/layout-<id>.json` without replacing
earlier evidence; a damaged existing manifest fails closed. A flat UUID segment
avoids nesting beneath legacy folders named `v2`; an exact overlap with a known
legacy basename requires manual reconciliation before any transfer.

The Settings caller previews the mapping, displays it with the saved manifest
path, and starts only after confirmation. IPC revalidates the acknowledgement and
preflights the complete selection in a blocking worker. Database locks end before
manifest persistence and provider I/O. Tests exercise actual LocalPath uploads,
mocked S3/WebDAV HEAD/PUT requests and exact bodies, legacy preservation, and
stable keys after root rename, restart, and staged snapshot restore. Provider
fixtures establish protocol behavior, not live-cloud or release qualification.

## Benchmark and acceptance plan

Benchmark production query and cache functions on reproducible 1k, 10k, 50k and 100k fixtures; add 500k where resources permit. Include metadata-heavy rows, sparse fields, multiple tags/albums, stacked files, cold/warm caches, concurrent indexing, and deep pagination. Record hardware, commit, dataset generation, query plans, database/index bytes, peak memory, startup-to-first-page time, IPC bytes and p50/p95 latency.

Compare against the same baseline and publish results in `docs/benchmarks/`. A redesign is accepted only when correctness tests pass and measured target workloads improve without material startup, write, memory, or recovery regressions. Synthetic loops that do not exercise production functions are not evidence. No engine change or unmeasured speedup is promised by this plan.
