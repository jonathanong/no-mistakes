# `postgres-no-generated-column-writes`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

See [context and motivating cases](postgres-no-generated-column-writes-context.md).

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

```yaml
rules:
  - rule: postgres-no-generated-column-writes
    options:
      importSpecifier: "@app/db"
      sqlInclude: ["db/**/*.sql"]
      triggerMaintainedColumns: [updated_at]
```

The configured `importSpecifier` limits executor calls to proven receivers from
that module; `executorNames` selects calls on those receivers. There are no direct
`schema` or `embedded` options.

`unanalyzableSql: advise` reports residual uncertainty in non-failing advisories.
`--include-sql-verdicts` adds [per-subcheck SQL verdicts](../postgres-sql-verdicts.md).

See [details](postgres-no-generated-column-writes-options.md).

## Valid example

```sql
-- db/schema.sql
CREATE TABLE items (
  id bigint PRIMARY KEY,
  note text,
  normalized_note text GENERATED ALWAYS AS (lower(note)) STORED
);
CREATE TABLE orders (id bigint PRIMARY KEY, status text, updated_at timestamptz);

-- db/write.sql
INSERT INTO items (id, note) VALUES ($1, $2);
```

## Counterexample

Run each counterexample independently in a SQL file under `db/`. Each includes
its own schema so a finding for one write cannot cover for the other.

```sql
CREATE TABLE items (
  id bigint PRIMARY KEY,
  note text,
  normalized_note text GENERATED ALWAYS AS (lower(note)) STORED
);
CREATE TABLE orders (id bigint PRIMARY KEY, status text, updated_at timestamptz);

UPDATE items SET normalized_note = 'override' WHERE id = $1;
```

With `triggerMaintainedColumns: [updated_at]`, this also fails:

```sql
CREATE TABLE items (
  id bigint PRIMARY KEY,
  note text,
  normalized_note text GENERATED ALWAYS AS (lower(note)) STORED
);
CREATE TABLE orders (id bigint PRIMARY KEY, status text, updated_at timestamptz);

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
from which PostgreSQL computes it. For an `unanalyzable` finding, make the
executed SQL statically recoverable or set `unanalyzableSql: ignore`.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` (rule option, repository `effort`, or the invocation override) is not an error: the rule has no low implementation, so the check warns `requested effort low; effective effort max because this rule has no low implementation` and runs at `max`. At max it evaluates executor calls in TS/JS (helpers, builders, and up to 16 finite SQL alternatives per call) and SQL files whose writes are checked. Two scopes apply: `sqlInclude` (default `**/*.sql`) selects the schema inputs (table, column and trigger definitions) that establish which columns are generated, while `options.include` selects the query inputs whose writes are checked (when empty, the default DML paths apply).

### Unanalyzable inputs

Valid PostgreSQL can still be unsupported by this analyzer. When it cannot
recover statements such as `ALTER DEFAULT PRIVILEGES`, `CREATE PUBLICATION`,
or `REFRESH MATERIALIZED VIEW`, it diagnoses them under
[`options.unanalyzable`](../migrations/postgres-schema-unanalyzable.md). The
same diagnostic also catches rejected syntax such as `CREATE RECURSIVE VIEW`
without its required explicit column list; it reports missing analyzer facts,
not a PostgreSQL validity verdict.

Following recall over precision ([how analysis works](../../README.md#how-analysis-works)), inputs this rule cannot prove are never treated as clean; they follow `options.unanalyzable`.

- Inputs this rule cannot prove: dynamic executor calls that could assign a generated or trigger-maintained column and cannot be ruled out. They follow `options.unanalyzable`:
  - `report` (default): fails as a finding, e.g. `executed SQL is not statically recoverable, so a write to a generated or trigger-maintained column cannot be ruled out`.
  - `advise`: a non-failing advisory with the same reason.
  - `ignore`: omitted.

  Definite violations remain findings under every policy. The legacy `unanalyzableSql: fail` spelling means `report`; `advise` and `ignore` keep their names.

SQL schema statements selected by `sqlInclude` also retain diagnostics when the
lenient parser cannot parse or recover them. Each diagnostic identifies the file,
statement line, and statement head, and follows the same `options.unanalyzable`
policy. Valid sibling statements remain available to the catalog. A real
`DISCARD ALL` statement does not trigger this recovery diagnostic. Existing file
and line suppressions apply; rewrite unsupported syntax into a supported static
statement or suppress the specific diagnostic with a reason.

For schema parse failures, the implicit default advises for any statement that
cannot be analyzed inside a recognized `DO`, PL/pgSQL function, or procedure body.
This includes malformed body statements: an advisory means analysis is incomplete,
not that PostgreSQL accepts the SQL. Successfully parsed body statements still
contribute facts and rule findings. At the top level, complete recognized unsupported
forms advise; malformed and unrecognized statements retain findings. Recognition
covers selected `ALTER DEFAULT PRIVILEGES ... GRANT`, `CREATE PUBLICATION ... FOR`,
and `REFRESH MATERIALIZED VIEW` forms, not all PostgreSQL grammar. Explicit rule or
repository `unanalyzable` policies override these defaults. See the
[schema diagnostic migration](../migrations/postgres-schema-unanalyzable.md).

## Suppression

Use `no-mistakes-disable-next-line postgres-no-generated-column-writes` for a
known external table exception or an `unanalyzable` executor call, or the file directive for a migration utility
whose writes are validated elsewhere.

SQL recovered from an imported helper is reported in the helper's module at the helper's line. A directive in the helper's file suppresses it, and so does a directive in the calling file (file-level, on the executor call, or on the imported-helper call). See [Helper-line attribution and suppression](../postgres-facts.md#helper-line-attribution-and-suppression).

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
