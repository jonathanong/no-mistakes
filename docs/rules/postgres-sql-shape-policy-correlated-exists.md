# postgres-sql-shape-policy: Correlated EXISTS examples

`executorNames` replaces the default `[query, read, write]` list; it does not
extend it. Use `extraExecutorNames: [readPool, writePool]` to keep the defaults
and add names. Setting both options is a configuration error. When a replacement
omits a default name imported from `importSpecifier` and called in scoped files,
`check` emits a configuration warning naming the rule or project, the omitted
name, an example call site, and the fix. See
[executor selection](../configuration/rules.md#postgresql-executor-selection).

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `extraExecutorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

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
