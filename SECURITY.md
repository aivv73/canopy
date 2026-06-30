# Security and reliability scope

Canopy is currently an MVP/prototype. The implemented `cnp` command is local-only and uses readable JSON files under `.canopy/` as its storage format.

## MVP trust boundaries

- `.canopy/` is trusted local state. Anyone who can read or modify that directory can read or modify all Canopy data.
- File content classified as `secret` is stored in plaintext in `.canopy/virtual-tree.json` and `.canopy/workspace-ops.json`.
- Public/private separation is projection filtering only. It is not encryption, access control, sandboxing, or tamper resistance.
- `cnp projection materialize public ...` omits secret paths and unpublished changes from the materialized public tree and public history, but does not erase the private source data.

## Filesystem mutation rules

- `cnp file add` accepts only non-empty repository-relative virtual paths. Absolute paths, `..`, `.`, and paths containing `.canopy` are rejected.
- Materialization writes into a caller-provided directory. Non-empty unmarked directories are rejected.
- Canopy-created materialization directories contain a `.canopy-materialized` marker with an exact MVP marker value. Re-materialization clears only directories with that valid marker.
- The MVP marker is a safety guard, not a security boundary. Do not point materialization at valuable directories.

## Persistence limits

- JSON state writes use write-then-rename for individual state files.
- Cross-file updates are not transactional in this MVP.
- There is no schema migration, corruption recovery, or authenticated storage yet.

## Out of scope for this MVP

- Cryptographic privacy or encrypted local storage.
- Remote synchronization and remote authorization.
- Multi-writer conflict handling.
- Strong deletion guarantees for previously materialized files outside Canopy-managed directories.
