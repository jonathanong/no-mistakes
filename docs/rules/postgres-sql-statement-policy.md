# `postgres-sql-statement-policy`

Flags configured PostgreSQL statement kinds and settings in matching SQL files
and configured JavaScript/TypeScript executor calls. Use this to keep tests,
helpers, or seed files from changing shared database state or bypassing
constraints. The rule recognizes 23 statement kinds: `CREATE TABLE`,
`ALTER TABLE`, `CREATE INDEX`, `CREATE VIEW`, `TRUNCATE`, `DROP INDEX`,
`DROP VIEW`, `CREATE DATABASE`, `DROP DATABASE`, `ALTER DATABASE`,
`ALTER SYSTEM`, `CREATE SCHEMA`, `ALTER SCHEMA`, `DROP SCHEMA`,
`CREATE TRIGGER`, `DROP TRIGGER`, `CREATE FUNCTION`, `CREATE PROCEDURE`,
`DROP FUNCTION`, `DROP PROCEDURE`, `DROP TABLE`, `CREATE TYPE`, and `DROP
TYPE`. `CREATE UNIQUE INDEX` counts as `CREATE INDEX`; materialized views
count as `CREATE VIEW` / `DROP VIEW`; and `CREATE TEMP TABLE` counts as
`CREATE TABLE`. `ALTER TABLE` also covers `ENABLE` and `DISABLE TRIGGER`.
`CREATE OR REPLACE FUNCTION` and `CREATE OR REPLACE PROCEDURE` map to their
respective `CREATE` kinds.

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

`bannedStatements` defaults to the original seven kinds shown above,
preserving existing behavior. The case-insensitive `ddl` group expands to all
23 kinds listed above. Kind names ignore case and repeated whitespace. An
unknown kind or group is a configuration error.

Rule application `include`/`exclude` limits both sources; `sqlInclude` continues
to select only SQL files. `unanalyzableSql` defaults to `fail`; `ignore` skips unanalyzable calls, and `fail` reports
dynamic executor arguments and unparseable recovered SQL. No executor module
or factory is inferred from project conventions.

`bannedSettings` is an optional list of PostgreSQL configuration parameter
names and defaults to empty. Names match case-insensitively in `SET`, `SET
LOCAL`, `SET SESSION`, `set_config('name', ...)`, and `ALTER DATABASE` or
`ALTER SYSTEM ... SET` statements. `set_config` must be unqualified or called
through `pg_catalog`, and its first argument must be a static string literal.
Settings findings use target `setting:<name>`. Dynamic setting names are not
matched. Statement kinds and settings found in recoverable routine bodies or
static `EXECUTE` strings are checked like top-level SQL. If parsing fails,
recognized kinds and settings are still checked; `unanalyzableSql` controls any
remaining unanalyzable SQL.

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
      bannedStatements: [ddl]
      bannedSettings: [session_replication_role]
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
SET LOCAL session_replication_role = replica;
```

Fix: keep schema DDL in migrations, and inject failures through an executor
seam instead of changing shared schema or database settings.

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

Every configured banned statement kind and setting is a finding in included SQL
and configured executor calls, including statically recoverable statements
inside supported PL/pgSQL bodies and static `EXECUTE` strings.

## Options and defaults

`sqlInclude` defaults to `**/*.sql`. `bannedStatements` defaults to `CREATE
TABLE`, `ALTER TABLE`, `CREATE INDEX`, `CREATE VIEW`, `TRUNCATE`, `DROP INDEX`,
and `DROP VIEW`; `CREATE UNIQUE INDEX` and materialized views map to those
categories.
`ddl` expands to all 23 recognized kinds. `bannedSettings` defaults to `[]`
and matches literal setting names in `SET`, `SET LOCAL`, `SET SESSION`,
supported `set_config()` calls, and database/system `SET` statements. Kind and
group names are case-insensitive and ignore repeated whitespace; unknown kinds
and groups are configuration errors.

## Valid example

```sql
INSERT INTO foo (id) VALUES ('00000000-0000-0000-0000-000000000001')
  ON CONFLICT (id) DO NOTHING;
```

## Counterexample

```sql
CREATE TABLE foo (id uuid PRIMARY KEY);
SELECT pg_catalog.set_config('session_replication_role', 'replica', true);
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
