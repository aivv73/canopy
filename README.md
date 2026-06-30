# Canopy

Canopy is an experimental version-control system design, written toward a Rust CLI named `cnp`.

The project asks what a VCS could look like if it did **not** treat Git compatibility, raw object IDs, OS worktrees, or public-by-default history as foundational constraints.

## Product thesis

Canopy treats software development as a **policy-driven graph** of changes, semantic deltas, provenance, virtual file state, live workspace operations, projections, and capabilities.

A public repository is not the repository. It is one coherent, independently buildable **projection** of a richer permissioned graph.

## Core ideas

- **Change-level privacy** — visibility applies to changes, deltas, files, relations, lifecycle metadata, CI outputs, and provenance, not only to whole repositories.
- **Private files in shared repos** — files such as `.env` can be tracked safely; secret paths and content are hidden from unauthorized projections by default.
- **Merge embargoes** — security-sensitive work can be prepared privately, reconciled by its embargo owner, and disclosed later without leaking hidden history.
- **Virtual repository substrate** — Canopy does not make an OS filesystem checkout or Git-style worktree the core repository model.
- **Live replicated workspaces** — humans and agents can edit a shared workspace; live operations are durable process history but are not project history until promoted.
- **Semantic promotion** — workspace history is promoted into clean semantic deltas through proposal and policy-governed acceptance.
- **Provenance and references** — users and agents refer to issues, changes, named deltas, workspaces, and review threads through resolved composite references, not raw storage IDs.
- **Policy-governed crypto** — encryption domains, capabilities, key wrapping, signatures, CI access, retention, and disclosure follow policy boundaries.

## CLI direction

The CLI is `cnp`. It should expose semantic workflows instead of Git-shaped primitives:

```bash
cnp clone <url>                      # bootstrap trust and fetch a projection package
cnp history                          # show projection semantic history
cnp edit "OAuth cleanup"             # start/join a change-first edit session
cnp change propose                   # propose semantic deltas from workspace history
cnp change accept                    # accept a proposal under policy
cnp sync                             # transfer authorized data without changing visibility
cnp change publish "OAuth cleanup" --to public
cnp change disclose "SEC-2026-01" --to public
```

`clone` does not imply a filesystem checkout. Materialization and live workspace subscription are explicit operations.

## Design documents

- [`CONTEXT.md`](./CONTEXT.md) — Canopy domain glossary and canonical language.
- [`docs/design.md`](./docs/design.md) — narrative design overview.
- [`docs/cli.md`](./docs/cli.md) — CLI shape and example workflows.
- [`docs/mvp.md`](./docs/mvp.md) — local MVP scope, commands, and security limits.
- [`docs/adr/`](./docs/adr/) — architectural decisions captured during design.
- [`docs/agents/`](./docs/agents/) — agent-consumable repo workflow configuration.

## Status

This repository now contains a local-only MVP `cnp` CLI prototype plus design docs. The MVP demonstrates projection filtering, but it does **not** encrypt secrets: `secret` files are hidden from public projections while still stored in plaintext under `.canopy/`.

## License

MIT. See [`LICENSE`](./LICENSE).
