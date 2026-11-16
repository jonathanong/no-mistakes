---
model: claude-sonnet-5-5
runs: 3
max_turns: 10
timeout_seconds: 300
allowed_tools: [Read, Glob, Grep, Skill]
plugins: [../../plugins/no-mistakes-rules]
tags: [rules, rules-postgresql]
append_system_prompt: |
  There is no repository in this environment. Put your complete answer
  in your final message.
---


review this migration before i merge. the runner wraps every file in a
transaction; the table is created in this same file.

```sql
-- 0081_subscription_renewal_attempts.sql
CREATE TABLE subscription_renewal_attempts (
  id uuid PRIMARY KEY DEFAULT uuidv7(),
  subscription_id uuid NOT NULL REFERENCES subscriptions(id),
  attempted_at timestamptz NOT NULL DEFAULT now(),
  outcome text NOT NULL CHECK (outcome IN ('succeeded', 'declined', 'error'))
);

CREATE INDEX subscription_renewal_attempts_subscription_id_attempted_at_idx
  ON subscription_renewal_attempts (subscription_id, attempted_at DESC);

CREATE UNIQUE INDEX subscription_renewal_attempts_subscription_id_attempted_at_outcome_uniq
  ON subscription_renewal_attempts (subscription_id, attempted_at, outcome);

COMMENT ON TABLE subscription_renewal_attempts IS 'one row per renewal charge attempt';
```
