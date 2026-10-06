# `postgres-sql-statement-policy`

Flags configured PostgreSQL statement kinds in matching SQL files and configured
JavaScript/TypeScript executor calls. Use this
for config-driven or seed SQL that must not carry schema DDL (`CREATE TABLE`,
`ALTER TABLE`, `CREATE INDEX`, `CREATE VIEW`, `TRUNCATE`, `DROP INDEX`,
`DROP VIEW`). `CREATE UNIQUE INDEX` counts as `CREATE INDEX`. Materialized
views count as `CREATE VIEW` / `DROP VIEW`. Inserts and function bodies are
not findings unless those kinds are banned.

The rule uses shared schema facts (`extract_migration_facts`,
`collect_postgres_facts`) including direct statements peeled out of executable
PL/pgSQL `DO`, function, and procedure bodies and statically recoverable
`EXECUTE` strings, qualified or unqualified `format()` templates, and assigned
string variables. Routine bodies may be dollar, plain, escaped, Unicode (with
custom `UESCAPE`), or newline-concatenated strings.
Ordinary strings, comments, non-PL/pgSQL functions, and runtime-built
expressions remain inert. It does not re-parse SQL with a private parser.

```yaml
rules:
  - rule: postgres-sql-statement-policy
    scope: repository
    options:
      sqlInclude: ["backend/data-stores/psql/config-driven/**/*.sql"]
      bannedStatements:
        - CREATE TABLE
        - ALTER TABLE
        - CREATE INDEX
        - CREATE VIEW
        - TRUNCATE
        - DROP INDEX
        - DROP VIEW
```

`sqlInclude` defaults to `**/*.sql`. `bannedStatements` defaults to the list
above. Embedded SQL is opt-in through `importSpecifier` or explicit
`executorNames`. With `importSpecifier`, executor names default to `query`,
`read`, and `write`. `executorFactoryNames` enables local handles returned by
configured named factory imports, and `executorTypeNames` enables parameters
annotated with configured imported executor types. `trustedSqlTags` accepts
`{ module, name }` entries, as in `postgres-sql-shape-policy`.

Rule application `include`/`exclude` limits both sources; `sqlInclude` continues
to select only SQL files. `unanalyzableSql` defaults to `fail`; `ignore` skips unanalyzable calls, and `fail` reports
dynamic executor arguments and unparseable recovered SQL. No executor module
or factory is inferred from project conventions.

```yaml
rules:
  - rule: postgres-sql-statement-policy
    scope: repository
    include: ['**/*.test.ts', 'test-helpers/**']
    options:
      importSpecifier: '@example/db'
      trustedSqlTags: [{module: '@example/db', name: sql}]
      executorFactoryNames: [beginTransaction]
      executorTypeNames: [TransactionQuery]
      bannedStatements: [ALTER TABLE, CREATE TABLE, CREATE VIEW, TRUNCATE]
      unanalyzableSql: fail
```

For example, this test helper takes schema locks that can stall parallel tests:

```ts
import { query, sql } from '@example/db';
await query(sql`ALTER TABLE orders ADD CONSTRAINT reject_writes CHECK (false)`);
```

Inject failures through an executor seam instead:

```ts
const failingQuery: TransactionQuery = async statement => {
  if (statement.text.startsWith('UPDATE orders')) throw new Error('injected');
  return query(statement);
};
```

The same rule options work through the asynchronous Node `check()` API.

Counterexample: schema DDL in a config-driven file.

```sql
CREATE TABLE foo (id uuid PRIMARY KEY);
```

Fix: keep schema DDL in migrations, not in files this rule covers.

```sql
INSERT INTO foo (id) VALUES ('00000000-0000-0000-0000-000000000001')
  ON CONFLICT (id) DO NOTHING;
```

Use `no-mistakes-disable-next-line postgres-sql-statement-policy` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.

## Why and when

Use this rule on seed, config, or runtime SQL directories where schema DDL must
remain in migrations and the file's allowed purpose should be machine-checked.

## What it catches/requires

Every configured banned statement kind is a finding in included SQL, including
statically recoverable statements inside supported PL/pgSQL bodies.

## Options and defaults

`sqlInclude` defaults to `**/*.sql`. `bannedStatements` defaults to `CREATE
TABLE`, `ALTER TABLE`, `CREATE INDEX`, `CREATE VIEW`, `TRUNCATE`, `DROP INDEX`,
and `DROP VIEW`; `CREATE UNIQUE INDEX` and materialized views map to those
categories.

## Valid example

```sql
INSERT INTO foo (id) VALUES ('00000000-0000-0000-0000-000000000001')
  ON CONFLICT (id) DO NOTHING;
```

## Counterexample

```sql
CREATE TABLE foo (id uuid PRIMARY KEY);
```

## Fix

Move schema DDL to a migration directory, or narrow `sqlInclude` and the banned
set to match the file's intentional role.

## Suppression

Use `no-mistakes-disable-next-line postgres-sql-statement-policy` or
`no-mistakes-disable-file` for an approved exception, rather than silently
moving the file outside all policy coverage.

## Related rules

[`postgres-no-add-column`](postgres-no-add-column.md) governs migration column
shape; [`postgres-constraint-validate`](postgres-constraint-validate.md)
checks phased constraint rollout.
