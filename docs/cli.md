# Canopy CLI sketch

The Canopy CLI is `cnp`. It should make semantic workflows feel primary and keep Git-shaped implementation concepts out of the normal user model.

## Clone and materialization

```bash
cnp clone https://host/aivv/canopy
```

Default behavior:

- verify or bootstrap repository identity;
- verify projection signer and projection manifest;
- fetch a projection package;
- do not create a filesystem checkout;
- do not join a live workspace unless requested.

Explicit file materialization:

```bash
cnp materialize public/main ./canopy
cnp mount public/main ~/mnt/canopy
cnp sandbox public/main -- cargo test
```

## History and references

```bash
cnp history
cnp show change/oauth-cleanup
cnp show change/oauth-cleanup --provenance
cnp show change/oauth-cleanup --ops
```

`cnp history` shows projection semantic history, not raw operation logs or storage IDs.

Inspection output is human-facing. `cnp history` should identify the projection being viewed and show visible semantic deltas. `cnp show`/`cnp change show` should explain change identity, lifecycle, active editing association, visibility, and proposal state without turning raw workspace operations or storage identities into the primary model.

Canopy accepts human names, structured refs, and shell-friendly handles, resolving them internally to stable composite references.

## Change-first editing

```bash
cnp change start "OAuth cleanup"
cnp edit "OAuth cleanup"
```

`cnp edit` is a convenience workflow: find or create the change, join or create an attached workspace, materialize it locally, and capture edits as workspace operations.

Lower-level workspace commands remain available:

```bash
cnp workspace create "OAuth cleanup/live" --change "OAuth cleanup"
cnp workspace join "OAuth cleanup/live"
cnp workspace open "OAuth cleanup/live" ./oauth-cleanup
cnp workspace status
```

## Promotion

```bash
cnp change propose "OAuth cleanup"
cnp change accept "OAuth cleanup"
cnp change finish "OAuth cleanup"
cnp change abandon "Legacy refactor"      # stop an unaccepted intent
```

Promotion creates clean semantic deltas from workspace history. Acceptance is policy-governed. Finishing a change clears the active editing association used by file lifecycle commands; it does not accept, publish, disclose, delete, or compact the change.

## Visibility lifecycle

```bash
cnp sync
cnp change publish "OAuth cleanup" --to public
cnp change disclose "SEC-2026-01" --to public
```

- `sync` transfers authorized data without changing visibility.
- `publish` makes accepted semantic history visible to an audience.
- `disclose` reveals formerly private or embargoed information under policy.

Embargo convenience commands may wrap change policy and disclosure workflows:

```bash
cnp embargo create "SEC-2026-01"
cnp embargo prepare "SEC-2026-01" --against public/main
cnp embargo disclose "SEC-2026-01" --to public
```

## Status, checks, and conflicts

```bash
cnp status
cnp checks
cnp checks change/oauth-cleanup --logs
cnp conflicts
cnp conflict show conflict/oauth-issuer-rename
cnp conflict resolve conflict/oauth-issuer-rename
```

Status, CI checks, logs, artifacts, and conflict details are all policy-filtered.

`cnp doctor` is a diagnostic inspection view: it groups errors and warnings, offers selected next-action hints, and does not silently repair repository state.

## Policy, governance, and capabilities

```bash
cnp policy show
cnp policy explain change/oauth-cleanup
cnp policy propose ...
cnp governance history

cnp agent grant "fix tests" --change "OAuth cleanup" --paths src/auth --expires 2h
cnp capability derive --purpose ci --change "OAuth cleanup" --expires 30m
```

Capabilities are available to advanced users, but common agent and CI flows should expose safer high-level commands.
