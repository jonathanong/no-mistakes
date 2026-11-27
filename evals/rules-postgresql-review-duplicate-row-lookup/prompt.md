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


Review this PostgreSQL query before it merges.

```sql
CREATE TABLE members (
  id uuid PRIMARY KEY,
  display_name text NOT NULL,
  avatar_url text,
  deleted_at timestamptz
);

CREATE TABLE tickets (
  id uuid PRIMARY KEY,
  project_id uuid NOT NULL,
  assignee_id uuid REFERENCES members(id),
  reviewer_id uuid REFERENCES members(id),
  created_at timestamptz NOT NULL DEFAULT now()
);

SELECT t.id,
       (SELECT m.display_name FROM members m WHERE m.id = t.assignee_id AND m.deleted_at IS NULL) AS assignee_name,
       (SELECT m.avatar_url   FROM members m WHERE m.id = t.assignee_id AND m.deleted_at IS NULL) AS assignee_avatar,
       r.display_name AS reviewer_name
FROM tickets t
LEFT JOIN members r ON r.id = t.reviewer_id AND r.deleted_at IS NULL
WHERE t.project_id = $1
  AND (t.created_at, t.id) < ($2, $3)
ORDER BY t.created_at DESC, t.id DESC
LIMIT 50;
```
