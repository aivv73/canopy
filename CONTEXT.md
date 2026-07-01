# Canopy

Canopy is a version-control domain centered on permissioned repository projections, change-level privacy, delta provenance, and virtual collaborative workspaces.

## Language

**Public projection**:
A coherent, self-consistent repository view for an unauthorised or public audience that never reveals the existence, structure, timing, or effects of unauthorised changes, while remaining independently buildable and mergeable.
_Avoid_: Public clone, filtered checkout

**Embargo owner**:
The authorised person or group responsible for preparing a private or embargoed change and reconciling it with the public projection before disclosure.
_Avoid_: Public contributor, downstream maintainer

**Merge embargo**:
A delayed-visibility integration whose owner must absorb conflicts with public work before disclosure, so public contributors never become responsible for conflicts caused by history they were not authorised to see.
_Avoid_: Hidden merge, private branch

**Integration workspace**:
A private workspace where an authorised audience reconciles one repository projection with another, such as preparing an embargoed change for public disclosure against the current public projection.
_Avoid_: Disclosure workspace, hidden worktree

**Minimum authorized provenance**:
The smallest set of provenance events that an authorised actor may see and needs in order to understand and resolve an integration conflict. Integration workspaces expose this by default rather than full conversation history.
_Avoid_: Full provenance, content-only merge view

**Provenance escalation**:
The layered process for exposing more than the structurally required provenance in an integration workspace: Canopy begins with structurally attached conflict provenance, an embargo owner may request additional context, and policy or reviewer approval gates sensitive disclosure.
_Avoid_: Automatic full context, author-only curation

**Canonical graph identity**:
An internal storage identity used by the Canopy engine for authorised graph operations. Canonical graph identities do not leave storage and are not part of the user-facing reference model.
_Avoid_: Public object ID, Git SHA

**Repository store**:
The persistence boundary for repository records such as metadata, changes, workspace operations, virtual-tree caches, projection packages, policy roots, trust material, and future manifests. A repository store preserves records and write-group invariants; it does not decide projection visibility or materialization behavior.
_Avoid_: Database choice, storage backend as architecture

**Storage format**:
The versioned shape and semantics of persisted repository records. A storage format covers required fields, optional fields, enum values, record meaning, and replay or projection interpretation.
_Avoid_: File extension, backend name

**Compatible storage change**:
A change to persisted repository records that existing supported readers can load safely without changing the meaning of existing records. In the MVP, compatible storage changes are limited to additive optional fields with safe defaults.
_Avoid_: Serde happens to parse it, silent semantic change

**Storage migration**:
An explicit operation that transforms persisted repository records from one storage format to another. Migration is separate from diagnostics; `doctor` reports consistency problems but does not rewrite state.
_Avoid_: Automatic repair, implicit upgrade

**Projection identity**:
An identity that names semantic history within a specific projection rather than a storage object in the canonical graph. Projection identities remain tied to their projection's published meaning; disclosure publishes new semantic history instead of silently changing what an existing projection identity means.
_Avoid_: Global public ID, storage alias

**Projection view**:
A computed audience-specific semantic view of repository state. A projection view can drive history inspection and concrete renderings such as materialization entries, but it is not stored as canonical state in the MVP.
_Avoid_: Stored projection cache, checkout snapshot

**Named reference**:
A user-facing reference to a meaningful entity such as a change, delta name, issue, workspace, or review thread rather than an exposed storage identifier.
_Avoid_: Raw object ID, SHA-like user API

**Composite reference**:
A user-facing reference that identifies work by combining meaningful anchors such as an issue, change, named delta, workspace, or review thread. Composite references are the preferred way for humans and agents to point at code intent without exposing raw storage IDs.
_Avoid_: Bare object ID, SHA reference

**Named delta**:
A human- or agent-named operation inside a change, used to describe a specific semantic edit without requiring users to address a storage object.
_Avoid_: Patch hunk, raw delta ID

**Resolved composite reference**:
A composite reference that stores stable projection-semantic anchors for its target while displaying human-readable names that can be updated safely. It should preserve what was meant at creation time even when names change.
_Avoid_: Live name lookup, conversation-only pointer

**Append-only disclosure**:
The default disclosure model where formerly private history becomes visible by publishing new semantic nodes after the existing public projection, rather than rewriting public past or changing existing projection identities.
_Avoid_: Public history rewrite, retroactive disclosure

**Lifecycle metadata**:
The set of event properties that describe an object's lifecycle, such as creation, modification, publication, integration, and disclosure. Each lifecycle event can have its own visibility rather than inheriting a single object-level visibility.
_Avoid_: Single timestamp, commit date

**Publication metadata**:
The lifecycle metadata exposed by default in a public projection, describing when and how semantic history became public without revealing earlier private lifecycle events.
_Avoid_: Authored-at by default, private timestamp

**Independent property visibility**:
The privacy rule that visibility applies to object properties and lifecycle events as well as to whole objects, allowing fields such as author, organization, timestamp, discussion, review, and delta graph to have different audiences or coarsening policies.
_Avoid_: Object-only privacy, all-or-nothing visibility

**Policy stack**:
The layered rule model that decides intended visibility and workflow behavior: repository defaults, projection or audience rules, change-specific overrides, and reviewer gates. The policy stack defines what should be disclosed, hidden, coarsened, or escalated.
_Avoid_: Single privacy flag, ad hoc policy

**Capability set**:
The authorization material that enforces what an actor can actually access, combining decryption keys with policy claims such as repository identity, audience, projection, workspace, purpose, and expiry. Capability sets are scope- and purpose-bound first, may additionally bind actor or device identity, and enforce the policy stack without being the domain policy language itself.
_Avoid_: Policy document, visibility rule, decryption key only

**Governance event**:
A versioned and auditable change to policy or disclosure behavior, especially when it affects what private information may become visible. Governance events record who changed the rule, what was changed, and what approval or reviewer gate applied.
_Avoid_: Silent policy edit, admin setting change

**Policy ownership**:
The hybrid responsibility model for policy changes: maintainers own repository defaults, change or embargo owners own change-specific requests, and sensitive disclosure gates require governed and audited approval.
_Avoid_: Single policy owner, unaudited override

**Governance audience**:
The restricted audience allowed to inspect private governance events, such as auditors or security maintainers. Governance events are private to this audience by default and may publish redacted forms after disclosure according to policy.
_Avoid_: Public audit log by default, maintainer-only assumption

**Redacted governance event**:
A public or broader-audience form of a governance event that preserves accountability without revealing private object details, sensitive reasons, precise timing, or unauthorised actors.
_Avoid_: Full audit event, hidden audit event

**Live workspace operation**:
A durable operation produced inside a workspace that belongs to repository state but is not projection history until it is promoted or published. Live workspace operations support real-time collaboration, rollback, and provenance without forcing every edit into published semantic history.
_Avoid_: Ephemeral edit, published delta

**Promotion**:
The act of turning selected live workspace operations into projection history for an audience. Promotion gives durable workspace work a published semantic meaning without exposing all intermediate live operations.
_Avoid_: Commit, checkpoint

**Retention policy**:
The policy dimension that decides how long workspace operations, provenance, scratch state, and published history are kept, compacted, archived, encrypted, or deleted. Retention is part of the domain model rather than a storage implementation detail.
_Avoid_: Garbage collection detail, storage cleanup

**Workspace history**:
The durable journal of live workspace operations, scratch state, agent exploration, and process provenance. Workspace history is process history, not projection history, and can have different retention rules from promoted project history.
_Avoid_: Repository history, commit history

**Canonical history**:
The promoted project history made from changes, deltas, and published provenance that carry a stronger retention contract than unpromoted workspace history.
_Avoid_: Full operation log, workspace scratch

**Retention contract**:
The storage and audit promise attached to a repository object or operation class. Promotion changes the retention contract by moving selected workspace operations from working material into project history.
_Avoid_: Best-effort storage, accidental permanence

**Policy-driven graph**:
The Canopy repository model viewed as a development graph whose visibility, disclosure, provenance, retention, and cryptographic enforcement are governed by policy rather than fixed VCS conventions.
_Avoid_: Git with new commands, plain object database

**Semantic delta**:
A clean promoted delta that represents the intended meaning of selected workspace work, rather than necessarily preserving the exact sequence of live operations that produced it.
_Avoid_: Raw operation chain, patch hunk

**Derived-from link**:
An optional provenance link from a promoted semantic delta back to retained workspace operations or process history. Derived-from links allow audit and explanation without forcing all intermediate live operations into canonical history.
_Avoid_: Mandatory operation preservation, hidden ancestry

**Promotion proposal**:
A proposed semantic delta or set of deltas derived from workspace history. Any authorised human, agent, or tool may propose one, but it enters canonical history only after acceptance under the applicable policy.
_Avoid_: Automatic commit, unchecked agent summary

**Promotion acceptance**:
The policy-governed act that makes a promotion proposal part of projection or canonical history. Acceptance records the responsible actor and may require review, tests, or governance depending on policy.
_Avoid_: Save, auto-publish

**Change**:
An intent and policy container for related work. A change can begin before code exists and can gather workspace operations, promotion proposals, accepted semantic deltas, provenance, retention rules, and disclosure lifecycle over time.
_Avoid_: Commit, patch only

**Primary change**:
The single change that owns a delta's lifecycle, policy, retention contract, and disclosure behavior. Other changes may reference the delta, but they do not own it.
_Avoid_: Multiple owning changes, ownerless delta

**Change abandonment**:
A terminal lifecycle event for an unaccepted change whose intent is consciously stopped. Abandonment preserves the fact that the intent existed while preventing the change from entering projection history.
_Avoid_: Delete change, erase draft, drop commit

**Corrective change**:
A change whose accepted semantic purpose is to correct an earlier accepted change without deleting, abandoning, or rewriting that earlier history. Corrective changes enter history through the normal proposal, acceptance, and visibility lifecycle.
_Avoid_: Revert commit, history rewrite, delete accepted change

**Reversal**:
A corrective change kind that counteracts the accepted effect of an earlier change while leaving the earlier change in history according to its visibility.
_Avoid_: Undo commit, erase change

**Supersession**:
A corrective change kind that replaces the intent or effect of an earlier accepted change with a newer accepted intent while preserving both changes as semantic history according to projection visibility.
_Avoid_: Force-push replacement, pretend the old change never happened

**Abandonment reason**:
A later explanatory classification for why a change was abandoned, such as superseded, cancelled, merged elsewhere, or obsolete. The reason explains the stopped intent; it is not the abandonment event itself.
_Avoid_: Status subtype, deletion cause, failure code

**Related change reference**:
A typed relationship from one change to another change's delta or outcome. Related change references express relevance without transferring lifecycle or policy ownership.
_Avoid_: Shared ownership, duplicate delta

**Core relation type**:
A built-in relationship between changes or deltas that carries Canopy engine semantics, such as dependency, conflict, supersession, duplication, extraction, or contextual influence.
_Avoid_: Freeform related link, repository-only ontology

**Repository relation extension**:
A repository-defined relationship annotation that can express local workflow meaning without changing Canopy engine semantics unless a policy explicitly maps it to a core relation type.
_Avoid_: Engine relation, untyped note

**Relation visibility**:
The policy-controlled visibility of a relationship between changes, deltas, or other repository objects. By default, a relation is visible only when both endpoints and the relation type are visible to the audience.
_Avoid_: Source-inherited visibility, target-inherited visibility

**Encryption domain**:
A cryptographic protection boundary for a class of repository data, such as canonical storage, workspace history, projection history, governance events, or private files. Different domains may use different keys, retention rules, and disclosure workflows.
_Avoid_: Single repository key, blob-only encryption

**Layered encryption domains**:
The Canopy cryptographic model where repository data is protected by multiple encryption domains aligned with policy boundaries rather than by one repository-wide key.
_Avoid_: Server-side ACL only, monolithic encryption

**Derived capability**:
A narrower capability produced from an existing capability set for a specific actor, agent, workspace, CI job, purpose, or time window. Derived capabilities reduce ambient authority while preserving enough access to perform the delegated task.
_Avoid_: Full credential sharing, ambient repository access

**Purpose-scoped capability**:
A derived capability constrained by why it may be used, such as integration, CI, review, disclosure approval, or materialization. Purpose scope lets Canopy grant access without turning every authorised actor into a general reader.
_Avoid_: Unscoped token, broad read access

**Forward revocation**:
The baseline revocation promise that a removed actor loses access to future objects after capability removal and key rotation, while data already observed or cached by that actor cannot be made secret again.
_Avoid_: Retroactive secrecy, pretend revocation

**Retention-assisted revocation**:
A stronger revocation workflow that combines capability removal, key rotation or rewrapping, and retention actions such as compaction, deletion, secure erase, or cryptographic erasure for retained data the revoked actor should no longer access.
_Avoid_: Key rotation only, access-list removal only

**Cryptographic erasure**:
The destruction of encryption material for an encryption domain or object class so encrypted retained data becomes unreadable to everyone who lacks another copy of the key. Cryptographic erasure is a retention action, not a way to make already-observed data secret again.
_Avoid_: Deletion guarantee, revoking memory

**Hierarchical keying**:
The key-management model where encryption domains use layered keys that can wrap or derive narrower keys for audiences, changes, objects, workspaces, or purposes according to policy. Hierarchical keying avoids both repository-wide over-sharing and unmanageable per-object key sprawl.
_Avoid_: Repository-wide key, mandatory per-object key

**Key wrapping**:
The practice of encrypting a narrower data key with a broader or recipient-specific key so access can be granted, rotated, or revoked without rewriting every encrypted object.
_Avoid_: Direct object sharing, plaintext key distribution

**Private file**:
A file object whose path, content, metadata, or lifecycle may be restricted by policy. Secret files such as `.env` default to hiding both path and content from unauthorised projections.
_Avoid_: Public path with encrypted blob by default, repository-wide secret

**File-class policy**:
A policy rule based on the kind of file, such as secret, config template, team-private document, generated artifact, or public source file. File-class policies decide whether path, content, metadata, retention, and surrogates are visible.
_Avoid_: One private-file rule, extension-only rule

**Surrogate file**:
A public or broader-audience file object that safely substitutes for a private file, such as `.env.example` standing in for a hidden `.env`. Surrogate files are explicit projection content, not leaked placeholders.
_Avoid_: Redaction marker, hidden-file stub

**Surrogate proposal**:
A proposed surrogate file or object for broader-audience projection, created manually or by an authorised agent or tool and accepted under policy. Surrogate proposals should be based on safe schemas or examples rather than automatically derived from secret values unless a policy-approved redactor is used.
_Avoid_: Automatic secret redaction, generated placeholder leak

**Policy-approved redactor**:
A reviewed transformation allowed to derive broader-audience content from private content under policy. Policy-approved redactors are exceptional because mistakes can leak secrets or private structure.
_Avoid_: Ad hoc redaction, regex masking by default

**Layered signature**:
An integrity proof applied at the appropriate domain layer, such as canonical objects, promotion acceptances, governance events, projection manifests, or capability grants. Layered signatures prove integrity without forcing every private actor signature into public projections.
_Avoid_: Single commit signature, unsigned projection

**Projection manifest**:
A signed, audience-scoped description of the semantic history, visible file or object digests, visible relation metadata, policy context, and repository identity needed to verify a coherent projection without learning hidden canonical graph structure.
_Avoid_: Raw object list, hidden graph proof

**Projection signer**:
The delegated identity or service that signs a projection manifest for an audience under repository policy. A projection signer may attest to public semantic history without exposing private author identities, private signing events, or hidden canonical graph structure.
_Avoid_: Original author signature by default, anonymous tamper proof

**Actor identity**:
The identity of a human, service, agent, CI job, or other authorised actor that may receive capabilities, perform operations, or appear in provenance according to policy. Actor identity is distinct from repository identity, projection identity, and internal storage identity.
_Avoid_: User account only, storage author field

**Unauthorized public observer**:
An actor limited to public projections and public network-visible metadata. Canopy's default privacy model is designed to prevent this actor from learning hidden content, existence, structure, timing, effects, or sensitive relations.
_Avoid_: Public clone user

**Curious hosting server**:
A storage or sync service that may observe encrypted repository data and operational metadata but should not be able to read private content by default. Canopy reduces trust in this server with encryption and signatures, while some metadata side channels may require optional hardening.
_Avoid_: Fully trusted remote, malicious maintainer

**Revoked collaborator**:
A former authorised actor who may possess previously observed data or cached capabilities. Canopy promises forward revocation by default, not retroactive secrecy for data the actor already learned.
_Avoid_: Forgetful user, erased observer

**Compromised scoped actor**:
An agent, CI job, tool, or integration that has a derived or purpose-scoped capability and may attempt to exceed its delegated purpose. Canopy limits this actor through capability scope, expiry, policy claims, and audit.
_Avoid_: Fully trusted automation, broad reader

**Optional hardening threat**:
A threat class not fully handled by Canopy defaults, such as traffic analysis or compromised local devices, that can be mitigated by stronger repository policy, client configuration, or operational controls.
_Avoid_: Default guarantee, ignored threat

**Maintainer authority boundary**:
The rule that maintainers may govern repository policy but cannot bypass cryptographic access controls for past private data without the required capabilities. Sensitive maintainer actions should be recorded as governance events.
_Avoid_: Maintainer omniscience, silent admin bypass

**Multi-party governance**:
An optional hardening policy requiring threshold approval from multiple authorised reviewers or governance actors before sensitive grants, disclosure, policy changes, or capability issuance take effect.
_Avoid_: Single-admin approval, informal review

**Operational metadata**:
The storage and sync metadata a hosting server may observe by default, such as encrypted object sizes, object counts, and transfer timing. Operational metadata excludes private names, audience labels, relation types, lifecycle metadata, and content unless policy explicitly permits disclosure.
_Avoid_: Private metadata, projection metadata

**Metadata hardening**:
An optional policy and implementation mode that reduces operational metadata leakage through techniques such as padding, batching, cover traffic, private information retrieval, delayed sync, or opaque audience labels.
_Avoid_: Default guarantee, content encryption only

**Repository identity**:
The stable cryptographic identity of a Canopy repository, independent of any hosting URL, local path, or storage object ID. Repository identity anchors trust in projection signers, policy roots, trust bundles, and capability grants.
_Avoid_: Hosting URL, remote name, content hash

**Trust bundle**:
The bootstrap material used by `cnp clone` to verify and enter a repository, including repository identity, projection signer information, policy roots, and optional invitation capabilities.
_Avoid_: Git remote, clone URL only

**Invitation capability**:
A capability-bearing invitation that lets an actor join a repository, projection, or workspace with scoped access. Invitation capabilities include or reference the repository identity so they cannot be silently replayed against another repository.
_Avoid_: Invite link only, bearer URL without identity

**Public bootstrap**:
The default clone path for a public projection when no prior trust bundle exists. Public bootstrap may use trust on first use for repository identity while warning on identity changes; private access still requires an invitation capability or verified trust bundle.
_Avoid_: Private TOFU, URL-as-identity

**Strict bootstrap**:
A hardened clone policy that requires a verified trust bundle, transparency log proof, or other out-of-band repository identity verification before accepting a repository or projection.
_Avoid_: Silent first-use trust, host-only verification

**Identity change warning**:
A client warning or refusal triggered when a previously known repository identity, projection signer, or policy root changes unexpectedly. Identity change warnings protect public bootstrap users from silent repository substitution.
_Avoid_: Remote moved message, automatic trust reset

**Projection package**:
The local result of cloning a projection: a verified projection manifest plus the visible objects needed for that audience, without implying a materialized filesystem or live workspace subscription.
_Avoid_: Checkout, worktree

**Materialization request**:
An explicit user or tool request to present a projection, workspace, or selected virtual tree as real files or a sandbox through a materialization adapter. Clone does not imply materialization by default.
_Avoid_: Automatic checkout, default worktree

**Workspace subscription**:
An explicit subscription to live replicated workspace state. Clone may make a workspace subscription available, but does not silently join live editing state unless requested or specified by an invitation capability.
_Avoid_: Checked-out branch, implicit live workspace

**Scoped invitation**:
An invitation capability that grants a scoped capability set, including repository identity, projection or workspace scope, purpose, expiry, materialization permission, and any rights to derive narrower agent or CI capabilities. A human-readable role label may explain the invite but is not the authority by itself.
_Avoid_: Role-only invite, unrestricted bearer invite

**Derivation right**:
A permission inside a capability set or scoped invitation that allows the holder to create narrower derived capabilities for agents, CI jobs, tools, or workspaces under specified limits.
_Avoid_: Implicit delegation, full credential forwarding

**CI domain**:
A policy and capability boundary for automated builds, tests, and checks for a particular audience or sensitivity level, such as public CI, team-private CI, or security CI.
_Avoid_: One CI pipeline, fully trusted CI

**Scoped CI job**:
A CI job treated as a compromised scoped actor by default, with purpose-scoped, expiry-limited capabilities and policy-controlled access to inputs, logs, caches, artifacts, and status outputs.
_Avoid_: Broad repo CI token, trusted build by default

**CI output policy**:
The policy dimension that controls what a CI job may publish, including logs, artifacts, cache keys, status checks, timing, failure messages, and annotations. CI outputs are treated as potential privacy leaks.
_Avoid_: Public logs by default, artifact afterthought

**CI status projection**:
The audience-specific view of CI status information. By default, a CI status inherits its CI domain visibility, while policy may separately control status name, existence, timing, failure category, logs, annotations, and artifacts.
_Avoid_: Global status check, public failure by default

**Redacted CI status**:
A broader-audience CI status that reveals only policy-approved information, such as a generic pending or unavailable state, without exposing private CI domain names, timing, failure causes, or hidden projection existence.
_Avoid_: Security CI failing, hidden status leak

**CI output channel**:
A policy-controlled stream or artifact produced by CI, such as logs, annotations, binaries, coverage reports, cache entries, or test summaries. Each channel has its own visibility, retention, redaction, and approval policy.
_Avoid_: Single CI log, public artifact bucket

**Private CI output**:
The default output classification for non-public CI domains. Private CI outputs inherit the CI domain visibility unless a policy-approved redaction or projection explicitly publishes a broader-audience form.
_Avoid_: Public logs, automatic artifact publishing

**CI redaction approval**:
A policy or reviewer gate that permits a CI output to be redacted and published to a broader audience. CI redaction approval is required because logs and artifacts can leak private paths, names, stack traces, timing, or generated content.
_Avoid_: Automatic log scrub, best-effort masking

**CI cache domain**:
The visibility and capability boundary for CI cache entries. CI caches inherit their CI domain by default, and public CI must not read private-domain caches.
_Avoid_: Shared global CI cache, cross-audience cache

**Projection-scoped cache key**:
A cache key that includes projection, CI domain, capability scope, or equivalent non-leaking domain separation so cache entries cannot cross audience or sensitivity boundaries accidentally.
_Avoid_: Branch-only cache key, path-only cache key

**CI cache retention**:
The retention policy for CI cache artifacts, including compaction, expiry, encryption, and deletion. CI cache artifacts are derived repository material and must not outlive their policy domain accidentally.
_Avoid_: Permanent build cache, storage optimization only

**CI input view**:
A signed and authorised repository view supplied to a scoped CI job as its input. A CI input view may be a projection, workspace, integration workspace, promotion proposal, or materialized virtual tree, and gives CI only the access required for that run.
_Avoid_: Arbitrary repo checkout, broad CI clone

**CI sandbox materialization**:
A materialization of a CI input view into an isolated build or test sandbox. CI sandbox materialization should respect capabilities, file-class policy, output policy, and retention policy.
_Avoid_: Worktree checkout, shared build directory

**Layered workspace operation**:
A workspace replication model with low-level convergent operations for virtual tree and content state, plus optional semantic edit and provenance layers that explain user or agent intent.
_Avoid_: Text-only CRDT, semantic-only edit log

**Convergent operation**:
A low-level replicated operation designed to converge across workspace participants, such as virtual tree edits, content edits, creates, deletes, moves, or renames. Convergent operations provide the reliable substrate beneath semantic deltas and provenance.
_Avoid_: Commit, non-replicated patch

**Semantic edit layer**:
An optional layer that records intent-aware operations such as symbol rename, extract function, or dependency update above the convergent operation substrate. Semantic edit records help promotion and provenance but are not required for replication correctness.
_Avoid_: Required CRDT primitive, raw text diff

**Conflict object**:
A first-class workspace object representing an unresolved semantic, integration, or policy conflict. Conflict objects record affected operations, participants, provenance, visibility, and resolution lifecycle while the low-level replicated state remains convergent.
_Avoid_: Conflict marker, silent merge winner

**Conflict resolution lifecycle**:
The lifecycle of a conflict object from detection through assignment, discussion, resolution proposal, acceptance, and possible promotion. Conflict resolution lifecycle is policy-controlled and may differ across workspace or integration contexts.
_Avoid_: Manual file edit only, one-shot merge resolution

**Semantic conflict**:
A conflict in intent or meaning, such as incompatible renames, edit/delete races, schema disagreements, or integration incompatibilities, even when low-level operations can converge mechanically.
_Avoid_: Text conflict only, CRDT divergence

**Operation authorization**:
The policy and capability check that determines whether a workspace operation may enter workspace history. Operation authorization evaluates claims such as action, path, file class, purpose, actor, and workspace rather than relying on workspace membership alone.
_Avoid_: Workspace member can edit everything, role-only permission

**Operation acceptance**:
The act of admitting a proposed workspace operation into durable workspace history after authorization, validation, and any required policy checks. Operation acceptance is distinct from promotion acceptance into projection or canonical history.
_Avoid_: Local edit equals accepted operation, promotion acceptance

**Workspace role label**:
A human-readable label such as contributor, reviewer, or agent that explains expected behavior in a workspace but does not itself grant authority without corresponding capabilities and policy.
_Avoid_: Role as permission, ACL group only

**Operation validation**:
The pre-acceptance check that rejects workspace operations violating policy, capability scope, file-class rules, size limits, path limits, or other invariants before they enter shared workspace state.
_Avoid_: Accept then hope, post-hoc only

**Operation quarantine**:
A holding state for suspicious or policy-sensitive proposed operations that should be retained for review or audit but not merged into shared workspace state until explicitly accepted.
_Avoid_: Silent rejection, shared bad operation

**Operation limit**:
A capability or policy constraint on workspace operations, such as rate limits, operation count limits, path scopes, content-size limits, file-class restrictions, or semantic-action limits. Operation limits reduce damage from compromised or buggy scoped actors.
_Avoid_: Unlimited agent edit, workspace-wide write token

**Offline operation queue**:
A local queue of workspace operations recorded while disconnected and later submitted for operation validation and acceptance. Offline queued operations are not automatically part of shared workspace history.
_Avoid_: Offline commits, automatic sync acceptance

**Offline reconciliation**:
The policy-controlled process for validating, accepting, quarantining, or routing offline operation queues into an integration workspace when a client reconnects. Offline reconciliation handles conflicts, stale capabilities, and policy changes since disconnection.
_Avoid_: Blind push, last writer wins

**History view**:
The default user-facing history shown by `cnp history`: projection semantic history made of visible changes and semantic deltas, not raw storage IDs or live workspace operations.
_Avoid_: Commit log, raw operation log

**Inspection view**:
A human-facing CLI view that explains repository state, change intent, projection history, or diagnostics without making raw storage identities the primary model. Inspection views are not canonical history and may summarize or omit details according to projection and safety rules.
_Avoid_: Raw database dump, object inspection

**Status view**:
A human-facing local inspection view that summarizes current repository state, active editing association, change lifecycle counts, workspace operation volume, and lightweight next-action hints. A status view is not a consistency audit and does not replace `doctor`.
_Avoid_: Doctor report, full audit, machine API

**Edit session**:
A user-friendly workflow that finds or creates a change, joins or creates an attached workspace, materializes it locally, and captures edits as workspace operations.
_Avoid_: Checkout, branch switch

**Publish command**:
A user action that makes accepted semantic history visible to an audience under policy. Publishing is distinct from syncing data and from disclosing formerly private or embargoed information.
_Avoid_: Push, upload

**Disclosure command**:
A user action that reveals formerly private or embargoed content, provenance, lifecycle metadata, or semantic history to a broader audience under disclosure policy.
_Avoid_: Publish, merge

**Sync command**:
A user action that transfers or updates projection packages, workspace operations, manifests, and other authorised repository data without necessarily changing audience visibility.
_Avoid_: Push, pull

**Shell-friendly handle**:
A stable, readable CLI handle for a resolved composite reference, such as `change/oauth-cleanup` or `delta/oauth-cleanup/tighten-redirect-uri-validation`.
_Avoid_: Raw storage ID, SHA
