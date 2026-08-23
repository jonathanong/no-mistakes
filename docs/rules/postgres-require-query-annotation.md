# `postgres-require-query-annotation`

Require a leading `/* name */` block comment on executed PostgreSQL SQL so
slow-query logs and `EXPLAIN ANALYZE` can name the statement. Line comments
(`-- name`) do not count. `BEGIN`, `COMMIT`, and `ROLLBACK` are exempt.

The rule uses the shared PostgreSQL embedded-SQL facts
(`extract_embedded_sql_from_source`, `collect_postgres_facts`) and
`sql_requires_query_annotation`. It does not re-parse TypeScript with a
private parser.

```yaml
rules:
  - rule: postgres-require-query-annotation
    scope: repository
    options:
      include: ["src/**/*.ts"]
      exclude: ["src/generated/**"]
      importSpecifier: "@data-stores/psql"
      executorNames: [query, read, write]
```

`importSpecifier` defaults to `@data-stores/psql`. `executorNames` defaults to
`query`, `read`, and `write`.

Counterexample: `query(\`SELECT id FROM posts\`)`.

```ts
import { query } from "@data-stores/psql";

export function list() {
  return query(`SELECT id FROM posts ORDER BY id DESC`);
}
```

Fix: put a non-empty block comment at the start of the executed SQL.

```ts
query(`/* posts/list */ SELECT id FROM posts ORDER BY id DESC`);
```

Use `no-mistakes-disable-next-line postgres-require-query-annotation` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.
