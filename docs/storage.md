# Repository store boundary

Status: storage-boundary notes for the local MVP. This document defines the
repository store contract before Canopy chooses SQLite, `redb`,
content-addressed storage, compact binary encodings, or any other backend.

## Boundary

The **repository store** is the persistence boundary for repository records:
metadata, changes, workspace operations, virtual-tree caches, future projection
packages, policy roots, trust material, and manifests.

The current implementation is `LocalStore`, a JSON-backed local store under
`.canopy/`. `LocalStore` is an implementation of the boundary, not the final
storage architecture.

The store owns:

- repository discovery;
- storage-format checks;
- record persistence;
- active-change metadata access;
- workspace-operation append semantics;
- private virtual-tree cache persistence;
- future transaction boundaries for named write groups.

The store does not own:

- projection filtering;
- relation or correction-link visibility;
- semantic replay rules;
- materialization filesystem safety;
- command output;
- workflow policy beyond primitive record updates and existence checks.

## Named write groups

The JSON MVP does not provide cross-file transactions. These write groups name
future atomicity boundaries without changing current behavior:

1. **Initialize repository**: create repository metadata, empty virtual tree,
   empty workspace operation log, and change-record location.
2. **Start change**: create a change record and set the active-change pointer.
3. **Start corrective change**: after command-layer validation confirms the
   target is accepted, create a change record with correction metadata and set
   the active-change pointer.
4. **Record file operation**: update the private virtual-tree cache and append a
   workspace operation.
5. **Finish change**: clear the active-change pointer for the named active
   change.
6. **Propose change**: derive a promotion proposal from workspace operations and
   mark the change proposed.
7. **Accept change**: set accepted status and acceptance timestamp.
8. **Publish/disclose change**: set the visibility timestamp.
9. **Abandon change**: mark an unaccepted change abandoned, clear the active
   pointer if needed, and rebuild the private virtual-tree cache.
10. **Rebuild private virtual-tree cache**: recompute and persist the cache from
    non-abandoned workspace operations.

## Append and cache semantics

Workspace operations are semantically append-only. The JSON MVP rewrites
`workspace-ops.json`, but callers use append semantics and future stores should
preserve operation-log behavior.

The private virtual tree is a replay-validated cache used for current private
materialization. It is not projection truth. `cnp doctor` compares the stored
cache with replay of non-abandoned workspace operations.

Public projection outputs must not read current private virtual-tree contents.
Public history and public materialization are derived from public-visible
accepted semantic deltas and visible relation metadata.

## Backend evaluation criteria

Future backend evaluation should compare candidates against these criteria
before selection.

### Correctness and atomicity

- Can the backend make the named write groups atomic?
- Can it preserve append semantics for workspace operations?
- Can it support replay validation and cache rebuilds without hidden side
  effects?
- Can it fail safely on partial writes, interrupted commands, and corruption?

### Inspectability and migration

- Can developers inspect local state during early Canopy development?
- Can records be exported to readable debug formats?
- Does the backend have a clear schema/version migration story?
- Can old MVP JSON repositories be imported or migrated deliberately?

### Distribution and dependency risk

- Does the backend add native dependencies or platform-specific packaging risk?
- Does it work across Linux, macOS, and Windows without unusual build steps?
- Is the operational model understandable for a local-first CLI?

### Performance

- Does the backend improve write-group durability or replay performance without
  forcing premature optimization?
- Does it handle operation-log growth and projection replay efficiently enough
  for the next local slices?
- Can performance work wait until correctness, trust, and storage invariants are
  stable?

### Future trust and sync needs

- Can the backend store policy roots, trust material, projection manifests,
  projection packages, and capability-related records without confusing them
  with public projection semantics?
- Can sync move repository records without depending on local file layout?
- Can encrypted or authenticated storage layers be added later without changing
  user-facing references?

## Deferred choices

Do not select SQLite, `redb`, binary encodings, or a `RepositoryStore` trait only
because they are plausible. The next backend choice should be made after the
store contract and write groups have proven stable under local workflow changes.
