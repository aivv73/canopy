# Policy changes are governed and separately enforced

Canopy separates the policy stack from capability enforcement. The policy stack defines intended visibility and workflow behavior through repository defaults, projection or audience rules, change-specific overrides, and reviewer gates, while capability sets enforce what an actor can actually access. Changes that affect disclosure or sensitive visibility are governance events with an audit trail; those events are private to a governance audience by default and may publish redacted forms after disclosure according to policy.
