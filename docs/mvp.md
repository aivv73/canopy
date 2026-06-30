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
- `cnp status` shows repository status, storage format, the active change, and captured workspace operation count.
- `cnp doctor` validates local JSON state and reports errors/warnings without repairing it.
- `cnp change start <name>` creates an active change.
- `cnp change finish <change>` clears the active editing association for the current change without deleting the change or changing projection history.
- `cnp change abandon <change>` marks an unaccepted change as abandoned intent history, hides it from default change lists, and removes its effects from current private materialization.
- `cnp change list [--all]|show|current|proposal` inspect changes and promotion proposals by named references. `--all` includes abandoned changes.
- `cnp file add <path> [--class public-source|config-template|secret]` records an explicit workspace operation and updates the private virtual tree cache.
- `cnp file update <path> [--class public-source|config-template|secret]` records an explicit update operation.
- `cnp file remove <path>` records an explicit removal operation.
- `cnp file rename <old-path> <new-path> [--class public-source|config-template|secret]` records an explicit rename operation.
- `cnp change propose <change>` creates semantic deltas from workspace operations.
- `cnp change accept <change>` accepts the proposal into project history.
- `cnp change publish <change> --to public` makes public-safe deltas visible in public history.
- `cnp change disclose <change> --to public` exists as the MVP shape for future disclosure semantics.
- `cnp history --projection public|private` shows semantic projection history.
- `cnp projection materialize public|private <out-dir>` writes a filesystem view explicitly.

## Change abandonment

`cnp change abandon <change>` stops an unaccepted change intent. Abandonment is not deletion: the change record, workspace operations, and any retained promotion proposal remain available for provenance. Default `cnp change list` hides abandoned changes; `cnp change list --all` shows them as `abandoned`, and `cnp change show <change>` can inspect them.

Only `active` and `proposed` changes can be abandoned in the MVP. Accepted, published, and disclosed changes cannot be abandoned; they need future revert or supersede workflows. Abandoned changes do not appear in public/private history because history remains accepted semantic history. Private materialization replays non-abandoned workspace operations so abandoned add/update/remove/rename effects disappear from the current private tree. Abandonment reasons such as superseded, cancelled, merged elsewhere, or obsolete are future work.

## Active change lifecycle

File lifecycle commands require an active change because workspace operations must belong to a change. `cnp change finish <change>` returns the repository to `Active change: none`; after that, file operations fail until another `cnp change start <name>` is run. Finishing a change is not promotion acceptance, publication, disclosure, deletion, or retention compaction. It only clears the local active-change pointer. Acceptance, publication, and disclosure do not automatically finish a change in the MVP; run `cnp change finish` explicitly when editing for that change is complete.

`cnp doctor` reports missing active-change references as errors and warns when an accepted or published/disclosed change remains active.

## Security warning

The MVP provides **projection filtering only**.

`secret` files are hidden from public history and public materialization, but their contents are stored in plaintext JSON under `.canopy/`. Do not use the MVP to store real secrets.

Cryptographic enforcement, encryption domains, signatures, capabilities, key rotation, revocation, and cryptographic erasure are future slices.

## Storage

The MVP stores readable JSON under `.canopy/` so the model can be inspected and changed quickly. JSON storage is temporary and may be replaced by SQLite, content-addressed storage, or another persistence layer later. The CLI rejects unsupported repository format values and reports missing or corrupt JSON state with the affected state file path. Cross-file updates are still not transactional.

The Rust implementation keeps JSON access behind `LocalStore` in `src/storage.rs`. Other modules should use that boundary rather than reaching into `.canopy/` directly. This is a refactor seam only; it does not introduce a durable migration framework or change the JSON schema.

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

`cnp doctor` checks local state for the MVP storage format, active-change references, workspace operation references, virtual path validity, virtual tree readability, and basic change readability. It reports errors and warnings and exits non-zero when errors are found. It does not automatically repair repositories.

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
