# Workspace history has policy-driven retention

Canopy treats live workspace operations as durable repository state, but not as projection or canonical project history until promotion. Workspace history is a process journal for collaboration, rollback, scratch state, and agent exploration, so its retention is governed by policy and may include compaction, archival, encryption, deletion, or secure erase. Promotion changes the retention contract by moving selected workspace operations into canonical history with stronger project-history guarantees.
