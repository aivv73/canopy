# Canopy MVP implementation map

The current implementation is a deliberately small Rust CLI under `src/`, with `main.rs` as a thin entrypoint.
It is a vertical slice for the first GitHub issues, not the target architecture.

## Command surface

- `cnp init [path]` creates `.canopy/` JSON state.
- `cnp change start|correct|finish|abandon|list|show|current|operations|preview|proposal|propose|accept|publish|disclose` manages and inspects a change-first workflow.
- `cnp file add|update|remove|rename ...` explicitly records file lifecycle operations against the active change.
- `cnp status` and `cnp doctor` inspect repository state and validate local JSON consistency.
- `cnp history --projection public|private` renders accepted history through projection rules.
- `cnp projection materialize public|private <out-dir>` writes a filtered tree to disk.

## Change abandonment

Abandoned changes are retained intent history for unaccepted work, not physical deletion. The MVP keeps `active` as the draft-like status name and adds `abandoned` as a terminal state for active/proposed changes. Default change lists hide abandoned changes, `--all` exposes them, proposals are retained for explanation, and private virtual tree replay excludes abandoned workspace effects. Accepted, published, and disclosed changes cannot be abandoned.

## Corrective changes

Accepted changes are corrected by new semantic changes rather than abandonment, deletion, or history rewrite. `cnp change correct <target-change> --kind reversal|supersession --name <name>` creates a new active change with optional correction metadata targeting an accepted change. Reversal counteracts an earlier accepted effect; supersession replaces an earlier accepted intent with a newer one.

Correction metadata explains semantic intent only. Materialization remains driven by accepted semantic deltas through projection visibility rules. Public history shows correction links only when both the corrective change and the corrected target appear in the same public projection history view.

## Inspection views

Change, history, and doctor inspection output is human-facing CLI explanation, not a machine-stable API or canonical storage dump. Inspection commands may summarize lifecycle, visibility, operation counts, and diagnostic hints, but they should keep semantic Canopy concepts primary and avoid exposing raw storage identities as the user model.

`cnp change operations` is a local workspace operation inspection view, not projection history or a raw storage dump. It may show local operation paths and classes while keeping raw operation IDs and content blobs out of primary user-facing output.

`cnp change preview` is a non-mutating promotion preview view. It derives the same semantic delta names that `cnp change propose` would store, but it must not create proposal data, change lifecycle state, or imply projection-history visibility.

Projection-specific inspection, especially public history, must use the same projection visibility rules as materialization: public output includes only accepted published/disclosed public-safe semantic deltas and must not reveal secret paths, hidden counts, or private-only effects.

## Active change lifecycle

The active change is a repository metadata pointer that decides where new workspace operations are recorded. `cnp change finish <change>` clears that pointer when the named change is currently active. Finishing does not change the change status, proposal data, accepted semantic deltas, publication/disclosure metadata, public/private history, or materialization semantics.

## Storage files

- `.canopy/repo.json`: repository metadata and active change handle.
- `.canopy/virtual-tree.json`: private full-tree cache for materialization.
- `.canopy/workspace-ops.json`: durable operation log captured by `cnp file add|update|remove|rename`.
- `.canopy/changes/*.json`: change records, optional correction metadata, promotion proposals, acceptance/publication timestamps.

## Repository store boundary

The repository store is the persistence contract. `LocalStore` is the current JSON-backed local MVP implementation of that boundary, not the final storage architecture. Backend selection for SQLite, `redb`, content-addressed storage, or compact encodings is deferred until the store contract and write groups are explicit.

The store owns repository discovery, storage-format checks, record persistence, workspace-operation append semantics, and future transaction boundaries. Projection filtering, relation visibility, materialization safety, command output, and semantic workflow orchestration stay outside the store boundary.

Within the `canopy-mvp-1` storage format, compatible changes are additive optional fields with safe defaults only. Unknown JSON fields may be ignored by the current MVP reader but are not a supported extension mechanism. Incompatible persisted-shape or semantic changes require a future storage format bump and explicit migration path.

Important write groups are: initialize repository; start change; start corrective change; record file operation; finish change; propose change; accept change; publish or disclose change; abandon change; rebuild private virtual-tree cache. The JSON MVP may update multiple files non-transactionally, but future stores should treat these as named groups when adding atomicity.

Workspace operations are semantically append-only even though the JSON MVP rewrites `workspace-ops.json`. The private virtual tree is a replay-validated cache for private materialization, not the authority for projection correctness.

## Projection model

- Private projection includes the private virtual tree and accepted private history.
- Public projection includes only accepted changes that were published or disclosed to `public`.
- Public projection filters out deltas and files classified as `secret` without redaction markers or hidden counts.
- Public materialization is reconstructed by replaying public-visible accepted/published semantic deltas. It must not read current private virtual-tree content for paths whose latest content, deletion, rename, or classification came from an unpublished/private change.
- The MVP private virtual tree is a cache/source for private materialization; it is not by itself a valid public projection snapshot once mutable file lifecycle operations exist.

Projection views are computed on demand. The projection/replay helpers own visibility decisions for projected history, materialization entries, correction links, and future relation visibility. Command modules format projection output; they should not independently decide which semantic deltas, correction links, or relation endpoints are visible.

The MVP keeps one documented asymmetry: private materialization renders the current private virtual tree, while private history remains accepted semantic history. Public projection outputs must be derived only from public-visible accepted semantic deltas and public-visible relationship metadata.


## Rust MVP module map

The Rust MVP is split by responsibility rather than by storage shape:

- `main` is the binary entrypoint: it parses CLI syntax and delegates command execution.
- `cli` owns Clap command syntax only.
- `model` owns serializable MVP domain types and small lifecycle predicates; command modules still enforce workflow transitions.
- `storage` owns the `LocalStore` JSON persistence boundary, repository discovery, storage format checks, and `.canopy/` file access.
- `paths` owns virtual path normalization and validation.
- `projection` owns public/private projection computation, including public semantic-delta replay and private virtual-tree replay that excludes abandoned effects.
- `materialize` owns filesystem materialization safety and writes already-computed entries; it does not decide visibility.
- `commands` owns command orchestration, split into focused change, file, history, status, and doctor command modules.

This split is behavior-preserving. It does not change the MVP JSON schema, command surface, projection rules, or materialization safety model.

### Responsibility boundaries

The module boundaries are intentionally narrow so later persistence and engine work can replace pieces without changing the public command seam:

- CLI syntax belongs in `cli`; command modules should not define new Clap shapes inline.
- User-facing workflow decisions belong in `commands`; `storage` should not print command output or decide lifecycle transitions.
- Persisted data shapes belong in `model`; schema changes should be reviewed as storage-format changes even when JSON remains the backing store.
- `.canopy/` file layout and JSON read/write behavior belong behind `LocalStore`; other modules should not construct repository state paths directly.
- Virtual path acceptance rules belong in `paths`; filesystem materialization must only consume already-normalized virtual paths.
- Projection visibility belongs in `projection`; `materialize` must not infer whether a file is public, private, secret, accepted, published, or abandoned.
- `doctor` should call shared helpers for replay and validation instead of owning independent projection or lifecycle rules.

### Current code layout

```text
src/
  main.rs                 # thin binary entrypoint
  cli.rs                  # Clap syntax
  model.rs                # persisted MVP data types
  storage.rs              # LocalStore JSON persistence boundary
  paths.rs                # virtual path normalization and validation
  projection.rs           # projection replay and materialization-entry computation
  materialize.rs          # marker-protected filesystem writes
  commands/
    mod.rs                # command dispatcher
    change.rs             # change lifecycle and promotion commands
    file.rs               # explicit file lifecycle operation commands
    history.rs            # projection history rendering
    status.rs             # lightweight repository status
    doctor.rs             # local consistency diagnostics
```

### Command flow

Most commands follow this direction of dependency:

```text
main -> cli -> commands -> storage/model/paths/projection/materialize
```

Projection materialization has a stricter flow:

```text
commands -> projection -> materialize
```

`projection` computes audience-visible entries. `materialize` only writes those entries into a marker-protected directory. This preserves the MVP rule that public materialization is derived from public-visible accepted/published semantic deltas, not from current private virtual-tree contents.

## Known MVP compromises

- JSON state keeps the implementation simple for the slice.
- There are no merges, remotes, identities, capabilities, or encryption.
- Cross-file persistence is not transactional.
- The current binary is split into file-level modules, but it remains a single local-only CLI crate rather than the target Canopy engine architecture.

## Technology timing

Canopy chooses infrastructure libraries only after the domain boundary they support is explicit enough to evaluate. Near-term work should prioritize store, replay, projection, materialization, and inspection correctness over premature choices for binary formats, network protocols, CRDTs, or capability DSLs. See [`docs/adr/0018-defer-infrastructure-choices-until-domain-boundaries-are-stable.md`](./docs/adr/0018-defer-infrastructure-choices-until-domain-boundaries-are-stable.md).
