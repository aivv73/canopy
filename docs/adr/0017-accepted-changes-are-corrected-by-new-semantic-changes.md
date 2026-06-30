# Accepted changes are corrected by new semantic changes

Accepted Canopy changes are not abandoned, deleted, or rewritten when their accepted effect later turns out to be wrong. Canopy models correction as a new change whose semantic purpose is to correct an earlier accepted change. The earlier change remains part of projection history according to its visibility, and the corrective change enters history through the normal proposal, acceptance, publication, and disclosure lifecycle.

The correction umbrella has at least two kinds. A reversal counteracts the prior accepted effect. A supersession replaces the prior intent or effect with a newer accepted intent. The MVP should target a whole change first and leave delta-level correction links for future work. Correction metadata explains intent; materialized file state is still produced by replaying accepted semantic deltas through projection visibility rules.

Public correction links are projection-scoped. A public history view may show that a corrective change targets an earlier change only when the target is visible in the same projection. If the target is private or otherwise invisible to the projection, public history must omit that correction link rather than exposing hidden history.

This preserves append-only disclosure semantics, keeps abandonment reserved for unaccepted intent, and avoids making Git-shaped revert commits or history rewriting the user model.
