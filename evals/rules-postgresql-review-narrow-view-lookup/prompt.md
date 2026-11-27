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
CREATE TABLE users (
  id uuid PRIMARY KEY,
  handle text NOT NULL,
  org_id uuid NOT NULL REFERENCES organizations(id),
  deleted_at timestamptz
);

CREATE VIEW user_directory AS
SELECT u.id,
       u.handle,
       o.name AS org_name,
       (SELECT count(*) FROM team_memberships tm WHERE tm.user_id = u.id) AS team_count
FROM users u
JOIN organizations o ON o.id = u.org_id
WHERE u.deleted_at IS NULL;

SELECT n.id,
       n.title,
       ud.handle AS author_handle
FROM notes n
LEFT JOIN user_directory ud ON ud.id = n.author_id
WHERE n.workspace_id = $1
  AND (n.created_at, n.id) < ($2, $3)
ORDER BY n.created_at DESC, n.id DESC
LIMIT 50;
```
