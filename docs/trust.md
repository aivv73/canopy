# Canopy trust model prep

Status: design boundary notes for future trust work. The local MVP does not
create keys, sign manifests, encrypt storage, evaluate capability tokens, or
perform clone/sync trust checks.

## Trust anchors and identities

Canopy's first trust anchor is **repository identity**: the stable
cryptographic identity of a repository, independent of hosting URL, local path,
content hash, or internal storage object. Repository identity anchors projection
signer delegation, policy roots, trust bundles, and invitation capabilities.

**Projection identity** controls visible semantics for an audience. It names
semantic history inside a projection and must remain separate from repository
identity and internal storage identity. A hosting URL can locate data, and a
storage/content identity can address internal objects, but neither is the trust
identity or user-facing reference model.

**Projection signers** are delegated under repository policy. A public
projection signer may differ from a private, security, or governance projection
signer, but each signer must be authorised for the projection or audience it
attests.

**Actor identity** names humans, services, agents, CI jobs, and other actors
that may receive capabilities, perform operations, or appear in provenance.
Actor identity is distinct from repository identity, projection identity, and
internal storage identity.

## Projection manifests

A **projection manifest** is a signed, audience-scoped description of a coherent
projection view. At concept level it may include:

- repository identity;
- projection identity and projection signer identity;
- policy context sufficient to verify the projection view;
- visible semantic history;
- visible file or object digests;
- visible relation metadata, including correction links only when both endpoints
  are visible in the same projection;
- manifest metadata needed for verification, freshness, or packaging.

A projection manifest must not reveal hidden canonical graph structure. Public
projection metadata must not reveal:

- hidden change existence;
- hidden correction or relation targets;
- private actor identities or private signing events;
- private lifecycle timestamps;
- hidden relation edges or hidden policy rules;
- private capability grants;
- hidden CI domains, statuses, logs, timings, artifacts, or failure categories;
- hidden file paths, file digests, or object identities;
- canonical storage graph shape.

The manifest concept is intentionally separate from a file format. Future work
may choose JSON, CBOR, or another encoding only after the manifest invariants
are stable.

## Bootstrap modes

**Public bootstrap** may use trust on first use for repository identity. A client
that first observes a repository identity through a public projection should
record it and warn if later bootstrap material claims a different identity for
the same configured source.

**Strict bootstrap** requires verified bootstrap material before accepting a
repository or projection. This may be a trust bundle, transparency log proof, or
other out-of-band verification.

Private access must not rely on unauthenticated first-use trust. It requires an
invitation capability or verified trust bundle that includes or references the
repository identity.

A **trust bundle** should conceptually include:

- repository identity;
- projection signer information;
- policy roots or policy-root commitments;
- optional invitation capabilities;
- any verification metadata required by the selected bootstrap policy.

An **invitation capability** is scoped bootstrap authority. It may grant access
to a repository, projection, workspace, or purpose-limited task. It must include
or reference repository identity so it cannot be replayed against another
repository. It may also include audience/projection/workspace scope, purpose,
expiry, actor or device binding, and derivation rights.

## Capability-token requirements

Canopy capability sets are enforcement material, not the policy stack itself.
They should be:

- repository-bound;
- projection, audience, workspace, path, change, or operation scoped as needed;
- purpose-bound first, because agent and CI actors may be ephemeral;
- expiry-aware;
- optionally actor-bound or device-bound;
- attenuable or derivable only toward narrower authority;
- auditable enough to support provenance and governance;
- evaluable without exposing hidden projection structure to unauthorised actors.

A future capability token system must support request-time context such as
operation, projection, path, change, actor, purpose, time, and policy claims. It
must also keep authorization separate from authentication: a token can carry or
reference actor claims, but it does not by itself prove who is at the keyboard or
on the device.

## Biscuit-auth evaluation

`biscuit-auth` is a strong candidate to evaluate later, not an MVP dependency.
It matches several Canopy needs:

- signed tokens that can be verified offline by holders of the root public key;
- attenuation by appending restrictive blocks;
- logic-rule authorization over facts and checks;
- request-time facts supplied by the verifier;
- token derivation that can narrow authority for agents, CI jobs, or workspaces.

Important fit questions and risks remain:

- Repository identity must be an explicit fact or root in every Canopy token.
- Projection, audience, workspace, purpose, expiry, and operation scope need a
  stable Canopy vocabulary before token schemas are chosen.
- Revocation is external to Biscuit's core model and would need Canopy-specific
  revocation lists, epochs, key rotation, or policy roots.
- Actor/device binding is possible as claims, but Canopy must define when it is
  required and how it is verified.
- Biscuit facts and Datalog rules must not leak hidden projection names,
  relation targets, file paths, policy rules, or storage graph shape.
- Biscuit authorisation policies must remain an enforcement representation, not
  the domain policy stack itself.

Recommendation: keep `biscuit-auth` on the candidate list for a future trust
experiment after capability vocabulary and projection manifest requirements are
stable. Do not add the dependency or design a Canopy capability DSL in the MVP.

## Local MVP non-goals

The local MVP does not implement:

- cryptographic repository identity;
- key generation, storage, rotation, revocation, or erasure;
- projection manifest generation or verification;
- layered signatures;
- encrypted local storage;
- capability-token parsing, attenuation, or evaluation;
- clone, sync, invitation acceptance, or trust-bundle verification;
- transport trust for WebSocket, HTTP, QUIC, or any other sync adapter.

The MVP provides projection filtering over plaintext JSON only. That is useful
for validating replay/projection semantics, but it is not cryptographic privacy
or access control.

## Future trust testing strategy

Future trust implementation should be tested through the highest available CLI
seam, following the current integration-test style. Tests should assert user
observable behaviour rather than cryptographic encoding internals.

Good future tests should cover:

- public trust-on-first-use bootstrap records repository identity;
- repository identity changes produce warnings or refusal according to policy;
- strict bootstrap accepts verified trust bundles and rejects mismatched ones;
- private invitations fail when repository identity does not match;
- projection manifests verify visible semantic history and visible file/object
  digests without revealing hidden metadata;
- public manifests omit hidden changes, hidden relation targets, private actors,
  private lifecycle timestamps, hidden file paths, hidden digests, and storage
  graph shape;
- capability checks reject operations outside scope, purpose, audience,
  projection, workspace, expiry, or derivation limits;
- derived agent or CI capabilities cannot broaden authority;
- revocation or epoch checks fail closed when the chosen trust model requires
  online or cached revocation data.

Tests should avoid asserting concrete key formats, token encodings, signature
algorithms, or third-party library internals unless the feature being tested is
specifically an interoperability boundary.
