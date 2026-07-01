# Canopy design overview

Canopy is a proposed VCS whose core model is a permissioned, policy-driven graph rather than a Git-compatible object database plus worktree.

## Repository model

A Canopy repository contains canonical storage, projection history, workspace history, provenance, policy, capabilities, and virtual file state. The public repository is only one projection of this graph.

A **public projection** must be coherent, independently buildable and mergeable, and must not reveal the existence, structure, timing, or effects of unauthorized history.

## Privacy and disclosure

Privacy is object-, property-, relation-, and lifecycle-aware. Visibility can differ for content, path, author, organization, timestamps, discussion, review state, CI outputs, relation edges, and provenance.

Embargoed work is reconciled by the embargo owner before disclosure. Disclosure is append-only by default: formerly private history becomes new public semantic history instead of rewriting the public past.

## Identity and references

Canonical graph identities are internal to storage. Users and agents use named, resolved composite references such as issues, changes, named deltas, workspaces, and review threads.

Projection identities name semantic history inside a projection, not storage objects in the canonical graph.

## Workspaces and promotion

Workspaces are live replicated editing surfaces over repository views. Workspace operations are durable process history, but not project history.

Promotion turns selected workspace work into semantic deltas through a promotion proposal. Promotion acceptance is policy-governed and may require review, checks, or governance approval.

Accepted changes are corrected by later semantic changes rather than abandonment, deletion, or history rewrite. A corrective change may reverse an earlier accepted effect or supersede it with a newer accepted intent, and it follows the normal proposal, acceptance, publication, and disclosure lifecycle.

## Policy and governance

The policy stack defines intended visibility and workflow behavior. Capability sets enforce access with cryptographic material plus claims such as audience, projection, workspace, purpose, and expiry.

Sensitive policy or disclosure changes are governance events. Governance events are private to a governance audience by default and may have redacted public forms.

## Cryptography and trust

Canopy uses layered encryption domains aligned with policy boundaries, hierarchical keying, key wrapping, purpose-scoped derived capabilities, and layered signatures.

Clone bootstraps trust in repository identity, projection signers, policy roots, and optional invitation capabilities. A hosting URL is not the repository identity.

Repository identity is the first trust anchor, while projection identity controls visible semantics for an audience. Projection signers are delegated under repository policy and sign audience-scoped projection manifests, not hidden canonical graph structure. User references remain projection-semantic; storage or content identities must not become the user-facing or trust identity model.

Capability sets are scope- and purpose-bound first, with optional actor or device binding. Public projection metadata must not reveal hidden change existence, hidden relation or correction targets, private actors, private lifecycle timestamps, hidden policy rules, private capability grants, hidden CI domains, hidden file paths or digests, or canonical storage graph shape.

The trust model prep notes in [`trust.md`](./trust.md) define the current vocabulary for projection manifests, bootstrap modes, capability-token requirements, and future trust testing. They are design boundaries, not an implemented crypto layer.

Canopy should not choose crypto, capability-token, storage, network, or live-collaboration libraries before the corresponding domain boundary is stable. The MVP prioritizes store/replay/projection correctness; future slices can evaluate storage backends, cryptographic primitives, capability systems, sync transports, and CRDTs against explicit Canopy invariants. Future sync semantics should be transport-agnostic: WebSocket, HTTP manifest exchange, QUIC, and local test adapters are transport options, not the semantic model.

## CI and automation

CI jobs and agents are treated as compromised scoped actors by default. They receive purpose-scoped, expiry-limited capabilities over signed input views. Outputs such as statuses, logs, artifacts, caches, timings, and annotations are policy-controlled because they can leak private state.


## MVP testing strategy

The first Rust implementation is tested through high-level CLI integration tests in `tests/mvp.rs`. Tests intentionally exercise the public command seam rather than private helper functions so the MVP can be refactored without changing user-facing behavior.

The current Rust MVP is a single local-only CLI crate split into responsibility modules. `cli` owns Clap syntax, `model` owns persisted MVP data shapes, `storage::LocalStore` owns `.canopy/` JSON persistence, `paths` owns virtual path validation, `projection` owns public/private replay and visibility computation, `materialize` owns marker-protected filesystem writes, and `commands` owns workflow orchestration and user-facing output. The split is an implementation boundary for the MVP, not the final engine boundary for replicated workspaces, capabilities, remotes, cryptographic storage, or policy enforcement.

Current coverage focuses on:

- initializing a local repository;
- starting, proposing, accepting, publishing, listing, and inspecting changes;
- inspecting repository status and promotion proposals;
- rendering human-stable workspace operation views for empty and mixed operation changes without raw operation IDs or content blobs;
- finishing active changes and verifying no-active-change command behavior;
- abandoning unaccepted changes while retaining intent history and replaying private state without abandoned effects;
- recording explicit `file add`, `file update`, `file remove`, and `file rename` operations with public/config-template/secret classes;
- running lightweight `doctor` diagnostics for healthy state and representative storage-format errors;
- omitting secret paths from public history and public materialization;
- including secret paths in private history and private materialization;
- preventing unpublished public-safe draft files and unpublished private updates to public paths from appearing in public materializations;
- covering accepted-change correction target validation, projection-safe correction links, and backwards-compatible optional correction metadata;
- rejecting invalid virtual paths and unsafe materialization targets.

This lightweight diagnostics coverage is not a substitute for later migration, recovery, remote-sync, authorization, authenticated-storage, or cryptographic privacy tests.
