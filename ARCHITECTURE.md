# Canopy MVP implementation map

The current implementation is a deliberately small Rust CLI in `src/main.rs`.
It is a vertical slice for the first GitHub issues, not the target architecture.

## Command surface

- `cnp init [path]` creates `.canopy/` JSON state.
- `cnp change start|propose|accept|publish|disclose` manages a change-first workflow.
- `cnp file add <path> --class <class>` explicitly records file operations against the active change.
- `cnp history --projection public|private` renders accepted history through projection rules.
- `cnp projection materialize public|private <out-dir>` writes a filtered tree to disk.

## Storage files

- `.canopy/repo.json`: repository metadata and active change handle.
- `.canopy/virtual-tree.json`: private full-tree cache for materialization.
- `.canopy/workspace-ops.json`: durable operation log captured by `cnp file add`.
- `.canopy/changes/*.json`: change records, promotion proposals, acceptance/publication timestamps.

## Projection model

- Private projection includes the private virtual tree and accepted private history.
- Public projection includes only accepted changes that were published or disclosed to `public`.
- Public projection filters out deltas and files classified as `secret` without redaction markers or hidden counts.

## Known MVP compromises

- JSON state and a single binary file keep the implementation simple for the slice.
- There are no deletes, moves, merges, remotes, identities, capabilities, or encryption.
- Cross-file persistence is not transactional.
- Splitting `src/main.rs` into storage, command, projection, and materialization modules is expected after the MVP grows beyond this slice.
