# Security and reliability scope

Canopy is currently an MVP/prototype. The implemented `cnp` command is local-only and uses readable JSON files under `.canopy/` as its storage format.

## MVP trust boundaries

- `.canopy/` is trusted local state. Anyone who can read or modify that directory can read or modify all Canopy data.
- File content classified as `secret` is stored in plaintext in `.canopy/virtual-tree.json` and `.canopy/workspace-ops.json`.
- Public/private separation is projection filtering only. It is not encryption, access control, sandboxing, or tamper resistance.
- `cnp projection materialize public ...` omits secret paths and unpublished changes from the materialized public tree and public history, but does not erase the private source data.
- Public materialization is reconstructed only from accepted and published/disclosed proposal data, never from unpublished private virtual-tree content.
- Inspection commands such as `cnp change show`, `cnp change current`, `cnp history --projection private`, and `cnp doctor` are local diagnostic views, not public-safe artifacts. They may reveal private metadata such as change names, lifecycle timestamps, proposal delta names, active editing state, and secret-class operation counts. Public-safe output must continue to use public projection rules.

## Filesystem mutation rules

- `cnp file add` accepts only non-empty repository-relative virtual paths. Absolute paths, `..`, `.`, and paths containing `.canopy` are rejected.
- Materialization writes into a caller-provided directory. Non-empty unmarked directories are rejected.
- Canopy-created materialization directories contain a `.canopy-materialized` marker with an exact MVP marker value. Re-materialization clears only directories with that valid marker.
- The MVP marker is a safety guard, not a security boundary. Do not point materialization at valuable directories.

## Persistence limits

Finishing a change clears only the local active-change metadata pointer. It does not delete stored workspace operations, semantic deltas, virtual-tree content, or plaintext secret data. Abandoning a change removes its effect from the current private virtual tree by replaying non-abandoned operations, but it still retains abandoned workspace operations and any secret content they captured in plaintext JSON. Abandonment replay refuses malformed non-abandoned workspace operation records rather than silently repairing or dropping them.


- JSON state writes use write-then-rename for individual state files.
- Cross-file updates are not transactional in this MVP.
- There is no schema migration, corruption recovery, or authenticated storage yet. The diagnostic command reports local consistency problems but does not repair them.

## Out of scope for this MVP

- Cryptographic privacy or encrypted local storage.
- Remote synchronization and remote authorization.
- Multi-writer conflict handling.
- Strong deletion guarantees for previously materialized files outside Canopy-managed directories.
