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


review this migration before i merge. our runner runs this file
without wrapping it in a transaction (no-tx mode).

```sql
-- 0042_events_account_created_idx.sql
-- events has ~80M rows. The index build cannot be split into smaller steps
-- and exceeds the 30s default, so statement_timeout is raised for this
-- build only and reset right after.
SET statement_timeout = '30min';

CREATE INDEX CONCURRENTLY IF NOT EXISTS events_account_created_idx
  ON events (account_id, created_at);

RESET statement_timeout;

COMMENT ON INDEX events_account_created_idx
  IS 'serves the account activity feed, keyset by (account_id, created_at)';
```
