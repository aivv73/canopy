# Local Canopy MVP

The first Canopy implementation slice is a local-only Rust CLI prototype named `cnp`.

Its purpose is to prove the core domain flow in the smallest possible form:

```text
local virtual tree
  + explicit workspace operations
  + change-first workflow
  + promotion proposal / acceptance
  + public/private projection filtering
```

## What the MVP supports

- `cnp init [path]` creates a local `.canopy/` repository.
- `cnp status` shows a local status view with repository format, active editing association, change lifecycle counts, corrective-change count, private workspace size, workspace operation volume, and lightweight next-action hints.
- `cnp doctor` validates local JSON state and reports grouped errors, warnings, and selected next-action hints without repairing it.
- `cnp change start <name>` creates an active change.
- `cnp change correct <target-change> --kind reversal|supersession --name <name>` creates an active corrective change targeting an accepted change without auto-generating file operations.
- `cnp change finish <change>` clears the active editing association for the current change without deleting the change or changing projection history.
- `cnp change abandon <change>` marks an unaccepted change as abandoned intent history, hides it from default change lists, and removes its effects from current private materialization.
- `cnp change list [--all]|show|current|operations|preview|proposal` inspect changes, workspace operations, promotion previews, and promotion proposals by named references. `list` presents a human change list view with active editing, lifecycle, role, and public visibility summary. `--all` includes abandoned changes in a separate section. `show` presents a human inspection view with identity, lifecycle, active-editing, operation-summary, visibility, and proposal sections. `operations` presents the workspace operations attached to one change without raw operation IDs or content blobs. `preview` shows semantic deltas that would be proposed without creating proposal data or changing lifecycle state.
- `cnp file add <path> [--class public-source|config-template|secret]` records an explicit workspace operation and updates the private virtual tree cache.
- `cnp file update <path> [--class public-source|config-template|secret]` records an explicit update operation.
- `cnp file remove <path>` records an explicit removal operation.
- `cnp file rename <old-path> <new-path> [--class public-source|config-template|secret]` records an explicit rename operation.
- `cnp change propose <change>` creates semantic deltas from workspace operations.
- `cnp change preview <change>` shows semantic deltas that would be proposed from workspace operations without mutating repository state.
- `cnp change accept <change>` accepts the proposal into project history.
- `cnp change publish <change> --to public` makes public-safe deltas visible in public history.
- `cnp change disclose <change> --to public` exists as the MVP shape for future disclosure semantics.
- `cnp history --projection public|private` shows semantic projection history with a projection header, lifecycle visibility fields, and visible semantic deltas.
- `cnp projection materialize public|private <out-dir>` writes a filesystem view explicitly.

## Change abandonment

`cnp change abandon <change>` stops an unaccepted change intent. Abandonment is not deletion: the change record, workspace operations, and any retained promotion proposal remain available for provenance. Default `cnp change list` hides abandoned changes; `cnp change list --all` shows them as `abandoned`, and `cnp change show <change>` can inspect them.

Only `active` and `proposed` changes can be abandoned in the MVP. Accepted, published, and disclosed changes cannot be abandoned; they need corrective changes. Abandoned changes do not appear in public/private history because history remains accepted semantic history. Private materialization replays non-abandoned workspace operations so abandoned add/update/remove/rename effects disappear from the current private tree. Abandonment reasons such as superseded, cancelled, merged elsewhere, or obsolete are future work.

A corrective change is new semantic work that targets an earlier accepted change and then follows the normal proposal, acceptance, and visibility lifecycle. Reversal counteracts an earlier accepted effect; supersession replaces an earlier accepted intent with a newer one. Public history shows correction links only when the target change is visible in the same projection. Correction metadata explains intent; materialization is still determined only by accepted semantic deltas visible in the requested projection.

## Active change lifecycle

File lifecycle commands require an active change because workspace operations must belong to a change. `cnp change finish <change>` returns the repository to `Active change: none`; after that, file operations fail until another `cnp change start <name>` is run. Finishing a change is not promotion acceptance, publication, disclosure, deletion, or retention compaction. It only clears the local active-change pointer. Acceptance, publication, and disclosure do not automatically finish a change in the MVP; run `cnp change finish` explicitly when editing for that change is complete.

`cnp doctor` reports missing active-change references as errors and warns when an accepted or published/disclosed change remains active.

## Security warning

The MVP provides **projection filtering only**.

`secret` files are hidden from public history and public materialization, but their contents are stored in plaintext JSON under `.canopy/`. Do not use the MVP to store real secrets.

Cryptographic enforcement, encryption domains, signatures, capabilities, key rotation, revocation, and cryptographic erasure are future slices.

Trust-model implementation is also out of scope for the MVP. The docs define repository identity, projection signers, projection manifests, trust bundles, invitation capabilities, and capability scope as future trust boundaries, but the local MVP does not generate keys, sign manifests, evaluate capability tokens, or encrypt local storage.

## Storage

The MVP stores readable JSON under `.canopy/` so the model can be inspected and changed quickly. JSON storage is temporary and may be replaced by SQLite, content-addressed storage, or another persistence layer later. The CLI rejects unsupported repository format values and reports missing or corrupt JSON state with the affected state file path. Cross-file updates are still not transactional.

The Rust implementation keeps JSON access behind `LocalStore` in `src/storage.rs`. Other modules should use that boundary rather than reaching into `.canopy/` directly. This is a refactor seam only; it does not introduce a durable migration framework or change the JSON schema.

Corrective-change metadata is an additive optional field in change records. Existing MVP JSON change records without correction metadata still load as non-corrective changes.

Within `canopy-mvp-1`, compatible storage changes are limited to additive optional fields with safe defaults. Unknown extra JSON fields may be ignored by the current MVP reader, but they are not a supported extension mechanism. Removing or renaming fields, adding required fields, changing enum meanings, or changing replay/projection semantics requires a future storage format bump. `cnp doctor` reports storage problems but does not migrate or repair state.

Future storage work should define repository-store and replay boundaries before choosing SQLite, `redb`, compact binary encodings, or other persistence technology. Inspectability, migration, and forward compatibility matter more than early encoding density in the MVP.

The repository store is the persistence contract; `LocalStore` is the current JSON-backed local implementation. The store owns repository records and write-group invariants, while projection/replay owns visibility and materialization owns filesystem writes. Workspace operations are semantically append-only even though the JSON MVP rewrites one file. The private virtual tree is a replay-validated cache, and `doctor` checks it against workspace operation replay.

Named write groups include repository initialization, starting changes, starting corrective changes, recording file operations, proposing, accepting, publishing/disclosing, finishing, abandoning, and rebuilding the private virtual-tree cache. Stronger atomicity for these groups is future backend work, not a guarantee of the JSON MVP.

See [`storage.md`](./storage.md) for the repository store boundary, write-group map, virtual-tree cache semantics, and future backend evaluation criteria.

## Projection and replay invariants

Projection views are computed on demand in the MVP; there is no stored projection cache. A projection view is the audience-specific semantic view used by inspection commands and by concrete renderings such as materialization entries.

Public projection outputs are derived only from public-visible accepted semantic deltas and public-visible relationship metadata. Correction links follow the same relation-visibility rule: public history shows a correction link only when both the corrective change and the corrected target appear in the same public history view.

The MVP keeps one asymmetry for local editing: private materialization renders the current private virtual tree, while private history remains accepted semantic history. `cnp doctor` validates local caches and metadata by calling shared replay/projection helpers rather than owning separate visibility rules.

## Implementation boundaries

The local MVP is organized as a single CLI crate with file-level modules:

- `cli` defines command-line syntax.
- `model` defines persisted MVP data types.
- `storage` owns `LocalStore` and `.canopy/` JSON persistence.
- `paths` validates virtual paths.
- `projection` computes public/private visibility and replay results.
- `materialize` writes already-computed projection entries into marker-protected directories.
- `commands` orchestrates user workflows and prints command output.

These boundaries preserve the command-line behavior covered by `tests/mvp.rs`; they are not the final Canopy engine architecture.

## Diagnostics

`cnp doctor` checks local state for the MVP storage format, active-change references, workspace operation references, virtual path validity, virtual tree readability, and basic change readability. It reports grouped errors and warnings, includes selected hints for lifecycle problems such as accepted or visible changes remaining active, and exits non-zero when errors are found. It does not automatically repair repositories.

## Inspection views

The MVP inspection commands are human-facing explanations rather than machine-stable APIs. `cnp change show` summarizes the change intent, lifecycle, active editing association, workspace operation counts, visibility, and promotion proposal. It keeps detailed workspace operation listings out of the default view while still surfacing secret-class operation counts for local awareness.

The human-stable output contract covers `cnp status`, `cnp doctor`, `cnp change list`, `cnp change show`, `cnp change operations`, `cnp change preview`, `cnp change proposal`, and `cnp history --projection public|private`. Human-stable output means section structure, labels, and important explanatory phrases are stable enough for users, docs, and tests; it does not make the text a machine-readable parser contract. Future machine-readable output should be introduced explicitly, such as with a future `--format json`.

`cnp change list` is a change list view: a local inspection index of change intent, not projection history, raw storage inventory, or a replacement for `cnp change show`. It groups the active editing change ahead of other changes, shows lifecycle status, primary/corrective role, and public visibility summary, and hides abandoned changes by default with a hint to run `cnp change list --all`. Correction targets, lifecycle timestamps, promotion proposal details, and workspace operation summaries belong in `cnp change show` or `cnp change proposal`.

`cnp change operations` is a workspace operation view: a local inspection view for workspace operations recorded for one change, not projection history, promotion proposal details, raw operation logs, raw JSON, patch previews, or machine API. It lists operation kind, path, rename target, and file class in recorded order while keeping raw operation IDs and content blobs out of normal primary UX.

`cnp change preview` is a promotion preview view: a non-mutating local inspection view for semantic deltas that would be proposed from current workspace operations, not a stored proposal, projection history, raw operation log, patch preview, content preview, or machine API. It does not create proposal data or change lifecycle state.

`cnp change proposal` is a promotion proposal view: a local inspection view for proposed semantic deltas and workspace derivation, not projection history, raw workspace operation audit, patch preview, or machine API. It shows proposed semantic delta names and summarizes derived workspace operation count while keeping raw workspace operation IDs out of normal primary UX.

`cnp status` is a lightweight status view, not a consistency audit. It summarizes local shape and always points users to `cnp doctor` for replay and storage consistency checks.

The status view may retain temporary legacy compatibility lines while the MVP evolves. Snapshot tests can protect those lines as current human-facing behavior, but they are not a long-term machine API guarantee.

`cnp history --projection public|private` remains projection semantic history: abandoned changes and raw workspace operations are not shown. Public history continues to omit secret semantic deltas and secret paths.

## Out of scope

- network clone/sync/remotes
- cryptographic privacy
- live multi-user workspace replication or CRDTs
- CI domains and scoped CI jobs
- governance approval workflows
- conflict objects
- metadata side-channel hardening
- FUSE/mount support
- Git compatibility

## Demo

```bash
cnp init demo
cd demo
printf "hello\n" > README.md
printf "SECRET=abc\n" > .env
printf "SECRET=\n" > .env.example

cnp change start "Initial project files"
cnp file add README.md
cnp file add .env --class secret
cnp file add .env.example --class config-template
cnp change propose "Initial project files"
cnp change accept "Initial project files"
cnp change publish "Initial project files" --to public

cnp projection materialize public ../public
cnp projection materialize private ../private
```

Expected public view:

```text
README.md
.env.example
```

Expected private view:

```text
README.md
.env
.env.example
```
