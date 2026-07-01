# Prior Art & Positioning Notes

Status: reference notes, not architectural commitments. These capture what Canopy
takes as inspiration from adjacent projects/critiques, and where Canopy deliberately
diverges or defers.

---

## 1. Atomic (atomicdotdev/atomic)

Rust, patch-theory based VCS, positioned for "agentic native development."
Apache-2.0, v0.5.1 as of Apr 2026. Module layout: `atomic-core`, `atomic-identity`,
`atomic-repository`, `atomic-remote`, `atomic-agent`, `atomic-config`.

### Take as reference (ideas, not dependency)

1. **Patch/change as a composable, commutative operation on a graph**, not a
   diff snapshot. Merging A then B equals B then A when independent; every
   change has a well-defined inverse; conflicts are data, not failures.
2. **Identity as its own crate/layer** (`atomic-identity`, Ed25519), not bolted
   onto domain model types.
3. **Virtual working copies / stacks** — agent work happens on isolated views
   of the same underlying graph, not divergent branches needing merge commits.
4. **AI/provenance as first-class** — session, turn, model, token cost,
   causal decision graph (goal → exploration → commitment → verification)
   recorded structurally, not as commit-message prose.

### Do NOT depend on `atomic-core`

Canopy diverges from Atomic on the inventory that matters most:

```
Atomic:
  patch theory / merge correctness / agent attribution & cost

Canopy:
  projection privacy / disclosure / policy graph / private-public
  semantic history
```

**Identity is the dangerous seam.** Atomic's change identity is hashed from
patch content/dependencies. Canopy's references must resolve to
projection-semantic anchors, not raw storage/content IDs. Adopting Atomic's
core would risk silently inheriting its identity model into a system where
identity visibility itself must be policy-governed — a bug class that's cheap
to avoid now and expensive to retrofit later.

### Process

1. Treat Atomic as prior art, not foundation.
2. Keep this doc as the canonical list of borrowed ideas vs rejected fusion.
3. No dependency on `atomic-core` until proven compatible.
4. Canopy's Projection/Policy model stays independent.
5. Revisit "do we need a patch-theory engine under semantic deltas" later,
   against a concrete test case (see below).

### Compatibility test (judge, not vibes)

> An abandoned change under a private embargo survives a rebase of hidden
> history, and the public projection still reveals neither its content nor
> the fact that it existed.

Any patch-theory engine (Atomic's, a fork, or one written in-house) is only a
candidate foundation if it passes this without leaking through its
identity/hash graph. If it can't, the question of reuse doesn't need further
discussion.

---

## 2. Theo (t3.gg) — Git critique / agentic VCS wishlist

Source: YouTube video, t3.gg, summarized from a transcript (timestamps approximate).

### Critique points and how Canopy maps to them

| Theo's point | Canopy's answer |
|---|---|
| Can't safely commit `.env`/secrets — history is forever, teams bolt on Doppler/Vault as glue | Change-level privacy + private files in shared repos — secret paths/content hidden from unauthorized projections by default |
| Security fixes are visible the moment they're committed, before release — attackers race patches | Merge embargoes — security-sensitive work prepared privately, reconciled by embargo owner, disclosed later without leaking hidden history |
| Access should be granular per-change, not per-repo | Visibility applies to changes/deltas/files/relations, not whole repositories — core Canopy thesis |
| Commits/branches are overhead for current dev pace; JJ's snapshot model is more ergonomic | Change-first workflow (`cnp change propose/accept`) instead of raw commit/branch primitives; Slopflow itself is already Jujutsu-backed |
| OS filesystem/disk is the wrong hot path — APFS fsync makes spinning up many small agent sandboxes slow (6-12s Ubuntu vs 31-140s on Apple silicon in his benchmark); proposes in-memory/Node-isolate-backed FS | **Not directly addressed yet** — see below |

### Where Canopy does NOT yet match the critique

Theo's filesystem-performance point is a different axis from privacy/policy:
it's about avoiding disk I/O hot paths for cheap, many parallel agent
sandboxes — not about who can see what.

Canopy's current "virtual repository substrate" addresses a *semantic*
question (OS checkout is not the source of truth; materialization is an
explicit operation) — not a *performance* question (does data live in
memory to avoid disk I/O entirely).

```
Canopy MVP:
  OS checkout is not source of truth
  materialize is an explicit operation
  virtual-tree.json is a local persisted JSON cache

Theo's desired VFS:
  filesystem operations avoid the OS/disk hot path
  many agent sandboxes are cheap to spin up
  code can live in memory / isolate-backed FS
```

These are different levels of "virtual." Conflating them in positioning would
overclaim what the MVP does. **MVP virtual tree is a semantic architecture
boundary, not a performance optimization** — and this is a deliberate,
ongoing position through the Correctness/Trust/Storage phases below, not a
temporary gap waiting to be filled next.

---

## 3. Market check — does anything solve change-level privacy natively?

Searched directly for: per-file/per-change VCS privacy, granular secret access
without submodules, merge embargo / disclosure-after-release patterns.

**Finding: nothing found treats visibility as a native VCS primitive at the
change level.** Everything in this space is one of three shapes, all of
which are exactly the "glue" Theo's critique calls out:

1. **External secret managers** (Vault, Doppler, AWS Secrets Manager) — solve
   granular access well, but live entirely outside version control. Secrets
   never enter the repo; the VCS has no opinion about them.
2. **Encryption bolted onto Git** (git-crypt and similar) — encrypts specific
   files via git-attribute hooks, but visibility is controlled by whether the
   repo is "locked," an external operational step, not a property the commit
   graph itself understands. History still contains the encrypted blob
   forever; only the plaintext is gated.
3. **Repo/file-level access control layers** (enterprise platforms like
   Kiteworks, secret-scanning tools like GitGuardian) — RBAC and DLP applied
   on top of or around a repository, not a per-change visibility model
   inside the VCS's own data structure.

Direct supporting evidence: granular, multi-level access to encrypted secrets
*within* a repository is consistently described as difficult — secrets
managers are framed as the answer specifically *because* VCS tooling doesn't
model fine-grained access well, and bringing on or removing a collaborator
typically means re-encrypting everything rather than adjusting a policy on
existing history.

**Caveat:** this is two targeted searches, not a systematic survey. It
supports "this looks like real white space" — it does not support "we have
proven no one else is building this." Re-check periodically; this space is
moving fast (see Atomic, Diversion, re_gent — all from the past ~6 months).

**Why this matters for positioning:** Canopy's change-level privacy and
merge-embargo model isn't just "a nice feature nobody prioritized" — it's
solving a problem the *current* ecosystem actively works around with
external tooling, by design, because the underlying VCS data model doesn't
support it. That's a stronger pitch than "we added permissions to Git."

---

## 4. Roadmap (sequenced by what depends on what)

```
1. Correctness
   replay, projection filtering, abandonment, doctor

2. Trust
   identity, capabilities, encryption

3. Storage
   transactional store / WAL / content-addressing

4. Performance VFS
   lazy materialization, memory-backed workspaces, sandbox adapters
```

Rationale for ordering:

- **Correctness before Trust**: policy has nothing meaningful to protect if
  the underlying replay/projection semantics aren't proven first.
- **Trust before Storage hardening**: identity and capabilities determine
  what a transactional store even needs to keep atomic and for whom.
- **Storage before Performance VFS**: lazy/in-memory materialization only
  makes sense once it's clear which operations, policies, and projection
  guarantees it has to preserve — otherwise it just makes an unprotected
  surface faster.

Performance VFS (the direct answer to Theo's APFS/sandbox-cost critique) is
explicitly **phase 4, not deferred-by-oversight**. Positioning materials
should say this directly: Canopy already departs from Git's "working
directory = repository" assumption, but does not yet solve agent-sandbox
filesystem performance. That's a known, ordered gap — not an unknown one.
