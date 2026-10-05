# `postgres-no-generated-column-writes`

Columns added by schema-qualified `ALTER TABLE` statements use the same table
identity as DML writes. ALTER-added generated columns retain generated status
and take precedence over `triggerMaintainedColumns`.

Forbids DML that writes a PostgreSQL `GENERATED ALWAYS` column. PostgreSQL
rejects those assignments at runtime (`ERROR: cannot insert into column
...`). The rule collects generated columns from migration SQL through the
schema fact source, then matches parsed `UPDATE` / `INSERT` / `MERGE`
statements — not regexes — at TypeScript executor call sites and raw `.sql`
files.

```yaml
rules:
  - rule: postgres-no-generated-column-writes
    scope: repository
```

`sqlInclude` defaults to `**/*.sql`. There is no hardcoded `backend/` or
migrations root. Mixed migration files are usable: schema DDL inside `DO $$`
blocks is collected into the catalog, DML (`UPDATE` / `INSERT` / `MERGE`)
inside `DO $$` is reported like top-level DML (see statement lines below), and PostgreSQL 18 `VIRTUAL` generated
columns still populate the catalog. `include` selects DML files (`.ts`,
`.mts`, `.tsx`, `.js`, `.sql` when unset). `importSpecifier` /
`executorNames` select TypeScript call sites. No module is selected by default;
a configured module enables `query`, `read`, and `write`.

Tables that are not declared in SQL — for example application election
`voteTable` relations — must be listed in `extraGeneratedColumns`. This rule
does not scrape `voteTable:` literals.

```yaml
rules:
  - rule: postgres-no-generated-column-writes
    scope: repository
    options:
      sqlInclude:
        - "db/migrations/**/*.sql"
      include:
        - "src/**/*.{ts,sql}"
      importSpecifier: "@example/db"
      executorNames: [query, write]
      extraGeneratedColumns:
        - table: votes
          column: created_at
      triggerMaintainedColumns: [updated_at]
```

Counterexample: DML assigns a generated column.

```ts
import { write } from '@example/db'

write(`UPDATE items SET created_at = now()`)
write(`INSERT INTO items (id, created_at) VALUES ($1, $2)`)
write(`INSERT INTO items VALUES ($1, $2, $3)`)
write(`INSERT INTO items (id) VALUES ($1) ON CONFLICT (id) DO UPDATE SET created_at = now()`)
```

```sql
MERGE INTO items t
USING s ON t.id = s.id
WHEN MATCHED THEN UPDATE SET created_at = now()
WHEN NOT MATCHED THEN INSERT (id, created_at) VALUES (s.id, now());
```

Fix: omit the generated column and write the source column instead. PostgreSQL
computes `GENERATED ALWAYS` values from that source.

```ts
write(`INSERT INTO items (id, note) VALUES ($1, $2)`)
write(`UPDATE items SET note = $1`)
```

Use `no-mistakes-disable-next-line postgres-no-generated-column-writes` for a
one-off exception, or `no-mistakes-disable-file` when a whole file is an
intentional migration of generated values.

## Why and when

Use this rule when PostgreSQL generated columns are maintained by the database
and application writes must not try to supply their computed values.

## What it catches/requires

`UPDATE`, `INSERT`, or `MERGE` statements must omit generated columns discovered
from migration SQL or listed in `extraGeneratedColumns`. The same statements
must omit columns listed in `triggerMaintainedColumns`, including
`ON CONFLICT DO UPDATE SET updated_at = orders.updated_at`. That assignment
looks like a no-op, but a `BEFORE UPDATE` trigger still fires and bumps the
column. TypeScript executor calls and included SQL files are both analyzed
where configured. A column that is both generated and listed uses the
generated-column message.
Catalogs use the final CREATE/ALTER/DROP table state in migration-file and
statement order, including statically recovered routine DDL. Dropping and
recreating a table replaces its generated status and positional column order.

## Options and defaults

The rule compiles its options into two internal inputs, not nested YAML
objects: the schema catalog and embedded-SQL matcher. There are no direct
`schema` or `embedded` options.

- `sqlInclude` supplies the schema catalog's SQL-file globs. An omitted or
  empty list defaults to `**/*.sql`.
- `include` selects files containing DML. When omitted or empty, it analyzes
  `.ts`, `.mts`, `.tsx`, `.js`, and `.sql` files; otherwise its glob list is
  used.
- `importSpecifier` supplies the embedded-SQL matcher's import source. When
  omitted or empty, it has no module default.
- `executorNames` supplies the imported executor names that contain SQL. An
  omitted or empty list uses `[query, read, write]` only with a configured module.
- `extraGeneratedColumns` adds `{ table, column }` pairs to the generated
  column catalog. It defaults to an empty list.
- `triggerMaintainedColumns` defaults to `[]`. A non-generated column whose
  name matches an entry, on any table in schema SQL, is treated as set by the
  database. An empty name or a case-insensitive duplicate is a config error.
  An entry that matches no column is a stale finding. `[]` leaves today's
  generated-column findings unchanged.

### Executor configuration

Omitting both `importSpecifier` and `executorNames` is a configuration error.
Set `importSpecifier` to your database module or list `executorNames` explicitly.
Use `executorNames: []` without a module to select no executor calls and retain
SQL-file/native-SQL analysis where supported. See the
[executor migration](../migrations/explicit-postgres-executors.md).

| Option | Default | Behavior |
| --- | --- | --- |
| `importSpecifier` | Empty | Set explicitly to your database module to match its named imports. |
| `executorNames` | Empty without a module; `[query, read, write]` with a module | Without a module, only explicitly listed names match named imports from any module. |

With both options omitted, executor calls (including `.query`) are not scanned.
A configured module or explicit `query` enables `.query` members. A configured module also recognizes
its transaction helpers. Native SQL and recovered SQL-builder fragments retain
their existing scopes. See [the migration notes](../migrations/explicit-postgres-executors.md).

## Valid example

```sql
INSERT INTO items (id, note) VALUES ($1, $2);
```

## Counterexample

```sql
UPDATE items SET created_at = now() WHERE id = $1;
```

With `triggerMaintainedColumns: [updated_at]`, this also fails:

```sql
UPDATE orders SET status = 'paid', updated_at = now() WHERE id = $1;
```

The finding says “do not write trigger-maintained column `orders.updated_at`;
it is listed in triggerMaintainedColumns, so the database sets it — remove it
from the INSERT/UPDATE”. An unknown configured name produces
“stale triggerMaintainedColumns entry: `updated_at` matches no column in schema SQL”.
Columns introduced by `ALTER TABLE ADD COLUMN` count as schema columns.
Raw SQL writes are reported separately at each statement's first code line,
so line suppressions apply to the statement containing the write.
Dollar signs inside unquoted identifiers do not start quoted SQL strings or
absorb subsequent statements.

## Fix

Omit `updated_at` from the trigger-maintained write and let its trigger set it:

```sql
UPDATE orders SET status = 'paid' WHERE id = $1;
```

Remove the generated column from the write and provide only source columns
from which PostgreSQL computes it.

## Suppression

Use `no-mistakes-disable-next-line postgres-no-generated-column-writes` for a
known external table exception, or the file directive for a migration utility
whose writes are validated elsewhere.

## Related rules

[`postgres-no-add-column`](postgres-no-add-column.md) controls schema widening;
[`postgres-sql-statement-policy`](postgres-sql-statement-policy.md) controls
which SQL statement kinds are allowed in a file.
[`postgres-column-requires-trigger`](postgres-column-requires-trigger.md) checks that a trigger-maintained column
actually has its trigger. This rule only rejects writes to the listed names.

Catalog history preserves existing definitions for `CREATE TABLE IF NOT EXISTS`
and `ADD COLUMN IF NOT EXISTS`; these statements add definitions only when absent.

Migration files replay in directory order, then by the first numeric run in each
filename (for example, `2.sql` precedes `10.sql`). Table lifecycle state keeps
schema-qualified identities. Qualified DML matches that identity; unqualified
DML matches a unique surviving table name and skips ambiguous names. Findings
include the schema when multiple live tables share a name. An unqualified CREATE
can still supply a later qualified ALTER for that same unqualified table.
Unqualified ALTER and DROP select a temporary table first, then an exact or unique
surviving qualified relation. Ambiguous names leave the live catalog unchanged;
qualify the statement when multiple schemas contain the same table name.

Live table history excludes DDL inside dormant function/procedure definitions or
conditional PL/pgSQL branches. Top-level DDL and unconditional immediate DO DDL
remain visible; broad schema policy checks still inspect routine definitions.

`ALTER TABLE IF EXISTS` leaves absent tables absent. Temporary declarations may
use `GLOBAL` or `LOCAL` modifiers without changing lifecycle order. Dollar-quote
tags accept Unicode identifier characters, including `$café$` and `$東京$`.

Quoted identifier components containing dots remain distinct from qualification:
`public.orders` and `"public.orders"` have separate histories. Temporary tables
occupy the `pg_temp` namespace, shadow unqualified lookups, and leave permanent
tables intact when dropped. Exception-handler DDL is conditional and does not
change definite live table state.

Within a migration source, `ROLLBACK` restores its prior table state and
`ROLLBACK TO SAVEPOINT` discards changes after that savepoint. Committed DDL
remains part of the write catalog, including DDL executed by an immediate DO block.
`COMMIT AND CHAIN` and `ROLLBACK AND CHAIN` start a fresh tracked transaction for
subsequent statements. `ABORT AND CHAIN` has the same rollback behavior, including
optional WORK or TRANSACTION modifiers, while `AND NO CHAIN` ends the transaction.
In immediate PL/pgSQL blocks, plain `RETURN` keeps later DDL out
of definite live history. `RETURN NEXT` and `RETURN QUERY` continue execution
in set-returning routines.

`format()` statements with unresolved `%I` or `%s` relation names remain available
to broad schema policy checks, but their synthetic placeholder names do not
change live table history, even when assigned or copied through SQL variables.
`%L`, escaped `%%`, and format calls without runtime placeholders remain concrete.

`extraGeneratedColumns.table` uses PostgreSQL identifier spelling: unquoted
names fold to lowercase, and quotes preserve case or literal dots. Unqualified
entries select a temporary table first, then an exact or unique surviving
relation; ambiguous names are skipped. Qualified entries retain their selected
schema. Stale-entry checks inspect that same relation rather than a namesake.

Assignments inside conditional, loop or exception scopes invalidate static SQL
variables, so later EXECUTE statements do not replay an uncertain branch value.

The rule shares prepared PostgreSQL schema and statement facts with other enabled
checks. Its query-file `include` scope is independent of schema `sqlInclude`, and
each configured executor profile uses the shared parsed TS/JS program. Raw SQL
findings retain each inner statement's physical line, including statements inside
a multiline `DO` block, so line and next-line suppressions target that write.

## Migration ordering

SQL files that themselves define schema (`CREATE` / `ALTER` / `DROP TABLE`) are
matched statement by statement against the catalog as of that write, so an
`INSERT` that precedes a later `DROP TABLE` or ordinary recreation is still
checked against the generated definition. Other files (TypeScript, DML-only SQL)
use the final catalog. `CREATE TABLE ... AS`, `LIKE`, `INHERITS`, and partition
tables keep an unknown positional column order, so positional `INSERT` values are
not attributed to later `ALTER ... ADD COLUMN` generated columns. Quoted and
unquoted column names that differ in case are distinct during `ALTER` replay.
