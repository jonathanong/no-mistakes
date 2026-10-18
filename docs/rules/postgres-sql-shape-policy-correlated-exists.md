# postgres-sql-shape-policy: Correlated EXISTS examples

Counterexample: correlated `EXISTS` wrapping `UNION ALL`.

```sql
SELECT 1 FROM posts
WHERE EXISTS (
  SELECT 1 FROM topics WHERE topics.post_id = posts.id
  UNION ALL
  SELECT 1 FROM tags WHERE tags.post_id = posts.id
);
```

Fix: keep the set operation uncorrelated, then test membership — for example
`post_id IN (SELECT candidate.post_id FROM ( … UNION ALL … ) candidate)` — or
use a single-arm `EXISTS` without a set operation.

```sql
SELECT 1 FROM posts
WHERE posts.id IN (
  SELECT candidate.post_id FROM (
    SELECT topics.post_id FROM topics
    UNION ALL
    SELECT tags.post_id FROM tags
  ) candidate
);
```

An uncorrelated `EXISTS` around a set operation, including a FROM-less
`SELECT EXISTS (… UNION …) AS alias` scalar probe, is allowed. An inner
placeholder on every arm does not make a correlated wrapping `EXISTS` safe.

Use `no-mistakes-disable-next-line postgres-sql-shape-policy` immediately
before the `EXISTS`, or `no-mistakes-disable-line` on that line, or
`no-mistakes-disable-file` when a whole file is an intentional exception.

[Back to the rule](postgres-sql-shape-policy.md)
