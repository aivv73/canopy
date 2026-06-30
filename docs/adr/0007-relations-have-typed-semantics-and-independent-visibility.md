# Relations have typed semantics and independent visibility

Canopy deltas have one primary change that owns their lifecycle, policy, retention, and disclosure behavior; other changes may point to them through typed related-change references without taking ownership. Canopy provides core relation types with engine semantics and allows repository relation extensions for local workflow vocabulary. Relations are privacy-sensitive graph edges, so their visibility is policy-controlled and defaults to visible only when both endpoints and the relation type are visible to the audience.
