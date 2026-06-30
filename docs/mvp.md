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
- `cnp change start <name>` creates an active change.
- `cnp file add <path> [--class public-source|config-template|secret]` records an explicit workspace operation and updates the private virtual tree cache.
- `cnp change propose <change>` creates semantic deltas from workspace operations.
- `cnp change accept <change>` accepts the proposal into project history.
- `cnp change publish <change> --to public` makes public-safe deltas visible in public history.
- `cnp change disclose <change> --to public` exists as the MVP shape for future disclosure semantics.
- `cnp history --projection public|private` shows semantic projection history.
- `cnp projection materialize public|private <out-dir>` writes a filesystem view explicitly.

## Security warning

The MVP provides **projection filtering only**.

`secret` files are hidden from public history and public materialization, but their contents are stored in plaintext JSON under `.canopy/`. Do not use the MVP to store real secrets.

Cryptographic enforcement, encryption domains, signatures, capabilities, key rotation, revocation, and cryptographic erasure are future slices.

## Storage

The MVP stores readable JSON under `.canopy/` so the model can be inspected and changed quickly. JSON storage is temporary and may be replaced by SQLite, content-addressed storage, or another persistence layer later.

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
