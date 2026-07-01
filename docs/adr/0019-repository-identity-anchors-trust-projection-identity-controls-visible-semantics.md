# Repository identity anchors trust while projection identity controls visible semantics

Canopy trust starts from repository identity, not from a hosting URL, local path, content hash, or storage object ID. Repository identity is the stable cryptographic identity that anchors projection signer delegation, policy roots, trust bundles, and invitation capabilities.

Projection identity and projection signers remain separate from repository identity. A projection signer is delegated under repository policy for an audience or projection and signs a projection manifest: an audience-scoped description of visible semantic history, visible file/object digests, visible relation metadata, policy context, and repository identity. Projection manifests should let clients verify a coherent projection without learning hidden canonical graph structure.

User references remain projection-semantic references. Storage/content identities may exist inside the engine, but they must not become the user-facing reference model or the trust identity for a repository. This avoids inheriting a content-hash identity model that can leak hidden structure or make projection visibility a bolt-on concern.

Public bootstrap may use trust on first use for repository identity with warnings on identity changes, while private bootstrap requires an invitation capability or verified trust bundle. Capabilities are scope- and purpose-bound first, may additionally bind actor or device identity, and must include or reference repository identity so they cannot be replayed against another repository.

Public projection metadata must not reveal hidden change existence, hidden correction or relation targets, private actor identities, private lifecycle timestamps, hidden relation edges, hidden policy rules, private capability grants, hidden CI domains or results, hidden file paths or digests, or canonical storage graph shape. Future cryptographic libraries, capability-token systems, sync protocols, and manifest file formats should be selected only after these trust boundaries are stable.
