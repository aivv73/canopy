---
status: accepted
date: 2026-06-30
---

# Abandoned changes remain intent history

Abandoned changes are terminal lifecycle events for unaccepted change intents, not physical deletion.

## Context

In Canopy, a change is an intent container rather than a commit. A local MVP user needs a way to stop an unaccepted intent, but deleting the change would erase useful provenance about what was tried and why it stopped.

## Decision

The MVP keeps `active` as the draft-like status name for now and adds `abandoned` as a minimal terminal lifecycle state for unaccepted `active` or `proposed` changes. Abandoned changes are hidden from normal change lists, exposed by `change list --all`, remain inspectable by named reference, and may retain workspace operations and promotion proposals. Private virtual-tree replay excludes abandoned workspace effects. Accepted, published, and disclosed changes cannot be abandoned.

## Consequences

Abandonment does not delete change records, workspace operations, proposal data, secret content captured in plaintext MVP JSON, or any accepted projection history. Future abandonment reasons such as superseded, cancelled, merged elsewhere, or obsolete are separate from the MVP abandonment event. Corrective changes, retention compaction, and cryptographic-erasure workflows remain distinct concepts.
