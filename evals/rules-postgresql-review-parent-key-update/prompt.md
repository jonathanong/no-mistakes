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


Review this PostgreSQL transaction for correctness when requests
run at the same time. Each request supplies a different label
and code for the same collection. There are no other writers.

```sql
CREATE TABLE collections (
  id bigint PRIMARY KEY,
  code text NOT NULL UNIQUE
);
CREATE TABLE collection_labels (
  collection_id bigint NOT NULL REFERENCES collections(id),
  label text NOT NULL,
  PRIMARY KEY (collection_id, label)
);

BEGIN;
INSERT INTO collection_labels (collection_id, label)
  VALUES ($1, $2);
UPDATE collections SET code = $3 WHERE id = $1;
COMMIT;
```
