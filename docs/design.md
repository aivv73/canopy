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

## Policy and governance

The policy stack defines intended visibility and workflow behavior. Capability sets enforce access with cryptographic material plus claims such as audience, projection, workspace, purpose, and expiry.

Sensitive policy or disclosure changes are governance events. Governance events are private to a governance audience by default and may have redacted public forms.

## Cryptography and trust

Canopy uses layered encryption domains aligned with policy boundaries, hierarchical keying, key wrapping, purpose-scoped derived capabilities, and layered signatures.

Clone bootstraps trust in repository identity, projection signers, policy roots, and optional invitation capabilities. A hosting URL is not the repository identity.

## CI and automation

CI jobs and agents are treated as compromised scoped actors by default. They receive purpose-scoped, expiry-limited capabilities over signed input views. Outputs such as statuses, logs, artifacts, caches, timings, and annotations are policy-controlled because they can leak private state.


## MVP testing strategy

The first Rust implementation is tested through high-level CLI integration tests in `tests/mvp.rs`. Tests intentionally exercise the public command seam rather than private helper functions so the MVP can be refactored without changing user-facing behavior.

Current coverage focuses on:

- initializing a local repository;
- starting, proposing, accepting, publishing, listing, and inspecting changes;
- inspecting repository status and promotion proposals;
- finishing active changes and verifying no-active-change command behavior;
- recording explicit `file add`, `file update`, `file remove`, and `file rename` operations with public/config-template/secret classes;
- running lightweight `doctor` diagnostics for healthy state and representative storage-format errors;
- omitting secret paths from public history and public materialization;
- including secret paths in private history and private materialization;
- preventing unpublished public-safe draft files and unpublished private updates to public paths from appearing in public materializations;
- rejecting invalid virtual paths and unsafe materialization targets.

This lightweight diagnostics coverage is not a substitute for later migration, recovery, remote-sync, authorization, authenticated-storage, or cryptographic privacy tests.
