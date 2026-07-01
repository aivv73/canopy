# MVP storage format allows additive optional fields only

The `canopy-mvp-1` storage format may evolve only through additive optional fields with safe defaults. Existing supported readers must be able to load older records without changing their meaning. Removing persisted fields, renaming fields, changing required field semantics, adding required fields without defaults, changing enum values, changing replay semantics, changing projection filtering semantics, changing lifecycle semantics, changing materialization marker semantics, or changing persisted path validation semantics requires a storage format bump.

Unknown extra JSON fields may be ignored by the MVP because the current Serde-based JSON reader tolerates them. This tolerance is not an extension contract: unknown fields do not carry supported Canopy semantics and may be rejected by future validation or migration tooling.

`doctor` reports storage compatibility and consistency problems but does not migrate or repair repository state. Future migration or repair commands must be explicit operations separate from diagnostics.
