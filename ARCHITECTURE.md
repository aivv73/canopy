# Canopy MVP implementation map

The current implementation is a deliberately small Rust CLI in `src/main.rs`.
It is a vertical slice for the first GitHub issues, not the target architecture.

## Command surface

- `cnp init [path]` creates `.canopy/` JSON state.
- `cnp change start|finish|abandon|list|show|current|proposal|propose|accept|publish|disclose` manages and inspects a change-first workflow.
- `cnp file add|update|remove|rename ...` explicitly records file lifecycle operations against the active change.
- `cnp status` and `cnp doctor` inspect repository state and validate local JSON consistency.
- `cnp history --projection public|private` renders accepted history through projection rules.
- `cnp projection materialize public|private <out-dir>` writes a filtered tree to disk.

## Change abandonment

Abandoned changes are retained intent history for unaccepted work, not physical deletion. The MVP keeps `active` as the draft-like status name and adds `abandoned` as a terminal state for active/proposed changes. Default change lists hide abandoned changes, `--all` exposes them, proposals are retained for explanation, and private virtual tree replay excludes abandoned workspace effects. Accepted, published, and disclosed changes cannot be abandoned.

## Active change lifecycle

The active change is a repository metadata pointer that decides where new workspace operations are recorded. `cnp change finish <change>` clears that pointer when the named change is currently active. Finishing does not change the change status, proposal data, accepted semantic deltas, publication/disclosure metadata, public/private history, or materialization semantics.

## Storage files

- `.canopy/repo.json`: repository metadata and active change handle.
- `.canopy/virtual-tree.json`: private full-tree cache for materialization.
- `.canopy/workspace-ops.json`: durable operation log captured by `cnp file add`.
- `.canopy/changes/*.json`: change records, promotion proposals, acceptance/publication timestamps.

## Projection model

- Private projection includes the private virtual tree and accepted private history.
- Public projection includes only accepted changes that were published or disclosed to `public`.
- Public projection filters out deltas and files classified as `secret` without redaction markers or hidden counts.
- Public materialization is reconstructed by replaying public-visible accepted/published semantic deltas. It must not read current private virtual-tree content for paths whose latest content, deletion, rename, or classification came from an unpublished/private change.
- The MVP private virtual tree is a cache/source for private materialization; it is not by itself a valid public projection snapshot once mutable file lifecycle operations exist.

## Known MVP compromises

- JSON state and a single binary file keep the implementation simple for the slice.
- There are no merges, remotes, identities, capabilities, or encryption.
- Cross-file persistence is not transactional.
- The current binary centralizes storage, path validation, projection, and materialization helper boundaries; a full file-level module split remains available as the MVP grows.
