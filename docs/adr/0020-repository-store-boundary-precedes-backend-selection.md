# Repository store boundary precedes backend selection

Canopy should define the repository store boundary before choosing SQLite, `redb`, content-addressed storage, compact binary encodings, or any other persistence backend. Backend shape must not become the architecture by accident.

The repository store is the persistence contract for repository records such as metadata, change records, workspace operations, virtual-tree caches, future projection packages, policy roots, trust material, and manifests. The store owns record persistence, repository discovery, storage-format compatibility, append semantics for operation logs, and named write-group invariants. It does not decide projection visibility, relation visibility, materialization filesystem safety, or CLI presentation.

`LocalStore` remains the current local MVP implementation. It is JSON-backed today and writes readable files under `.canopy/`, but callers should treat it as the local implementation of the repository store boundary rather than as a JSON-file convenience API. A Rust trait should wait until a second backend or prototype proves the abstraction; premature trait design risks freezing the wrong contract.

The store boundary should name write groups now, while stronger atomicity guarantees remain future backend work. Important write groups include repository initialization, starting a change, starting a corrective change, recording a file operation, proposing a change, accepting a change, publishing or disclosing a change, finishing a change, abandoning a change, and rebuilding the private virtual-tree cache. Future backends can then decide how to make those groups transactional without changing command, projection, or materialization semantics.

Workspace operations are semantically append-only even when the JSON MVP rewrites `workspace-ops.json`. The virtual tree is a replay-validated cache for current private materialization, not the source of projection correctness. Projection views remain computed on demand in the MVP; storing projection packages or manifests is future work.
