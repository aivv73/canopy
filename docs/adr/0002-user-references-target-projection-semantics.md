# User references target projection semantics, not storage objects

Canopy should not expose canonical graph identities as the normal user reference model. Canonical graph identities are internal to the engine, while users and agents refer to work through resolved composite references made from meaningful anchors such as issues, changes, named deltas, workspaces, and review threads. Projection identities identify semantic history within a projection, and disclosure is append-only by default so existing public references keep their original meaning.
