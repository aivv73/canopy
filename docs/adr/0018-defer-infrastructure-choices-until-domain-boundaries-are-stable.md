# Defer infrastructure choices until domain boundaries are stable

Canopy should choose infrastructure libraries when the domain boundary they serve is explicit enough to evaluate their invariants. The near-term implementation should prioritize store, replay, projection, materialization, and inspection correctness over premature choices for binary formats, network protocols, CRDTs, or capability DSLs.

The current Rust MVP should keep its simple stack: `clap`, `serde`, `serde_json`, `anyhow`, `chrono`, `tempfile`, and `assert_cmd`. Low-risk near-term additions may include `thiserror` for typed error boundaries, `camino` for UTF-8 path handling, `insta` when CLI output becomes stable enough for snapshots, and `tracing` when storage/replay flows need structured diagnostics.

Storage evolution should first define a repository store boundary and replay invariants, then evaluate SQLite or `redb`. `redb` is attractive because it avoids C dependencies and simplifies distribution, but it should not be selected before the storage boundary and migration expectations are clear. Early binary formats such as `postcard` or `bincode` should be deferred because Canopy needs migration, inspectability, and forward-compatibility discipline before compact encodings.

Network and live-collaboration choices should also wait. `cnp sync` must be defined semantically before choosing protocols or transports. Future sync semantics must be transport-agnostic: authorized data transfer, projection manifests, capability validation, visibility preservation, partial object availability, reconciliation, acknowledgements, and object requests must not depend on WebSocket, HTTP, QUIC, or any other transport semantics.

When network sync becomes in scope, WebSocket is a practical default transport because of broad deployment compatibility, HTTP manifest/package exchange may exist for simple sync, local file or pipe adapters should support tests, and QUIC may be added later as an optimized transport. None of those transport choices should become the architectural truth of Canopy sync. Live workspace work should begin with an event log and replay model; CRDTs should be introduced only if concrete collaboration semantics require them.

Capability enforcement is important, but not an MVP dependency. `biscuit-auth` is worth evaluating before inventing a Canopy capability DSL because signed, offline-checkable capability tokens with logic rules are close to Canopy's policy direction. That evaluation belongs in a future trust/capability slice after the relevant domain model is stable.
