# `postgres-identifier-length`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

Flags identifiers a migration declares that PostgreSQL will silently truncate.
PostgreSQL stores at most 63 bytes (`NAMEDATALEN - 1`) and only emits a
NOTICE, so two long names that share a prefix collide in the database. The
schema snapshot cannot catch this: it already holds the truncated name.

The rule reads declared identifiers from the shared migration facts
(`extract_migration_facts` / `collect_postgres_facts`), including statements
peeled out of `DO $$` bodies. It does not parse SQL itself. Auto-derived
names such as `<table>_<column>_fkey` are out of scope, as are identifiers
that appear only inside embedded TypeScript SQL.

```yaml
rules:
  - rule: postgres-identifier-length
    scope: repository
    options:
      sqlInclude: ["db/migrations/**/*.sql"]
      maxBytes: 63
```

With no options, the rule still flags declared names longer than 63 bytes in
`**/*.sql`. That default is the generic PostgreSQL name-length limit.

## Why and when

Use this rule on checked-in migrations. A 64-byte index or constraint name is
stored under a shorter identity, and a later migration that repeats the same
prefix can collide with it.

## What it catches/requires

Every identifier the SQL declares must be at most `maxBytes` UTF-8 bytes
after unquoting (`"Order"` is `Order`) and after dropping a schema qualifier.
Declared names are table names, column names, inline and table `CONSTRAINT`
names, index names, trigger names, function and procedure names, view and
materialized view names and explicit view column names, enum type names, and the new name in `ALTER TABLE`
`ADD CONSTRAINT`, `ADD COLUMN`, `RENAME TO`, `RENAME COLUMN … TO`,
`RENAME CONSTRAINT … TO`, and `ALTER INDEX … RENAME TO`.

A name that is only referenced is not a finding (`REFERENCES some_long_name`).
Length is measured on the unquoted identifier, including decoded PostgreSQL
Unicode identifiers (`U&"..."` with optional `UESCAPE`). Declaration modifiers,
nested comments, and recovered static `EXECUTE` statements retain their source
locations. `RECURSIVE VIEW` names and explicit columns are retained even when the SQL
parser cannot construct that view AST. Only omitted-language or explicit
`plpgsql` DO bodies contribute recovered PostgreSQL declarations; other
languages remain opaque. `DO LANGUAGE` bodies use their original
source lines. Routine-body declarations do not consume an outer declaration's line.
At the default 63-byte limit, the finding shows PostgreSQL's truncated prefix
on a character boundary. A smaller `maxBytes` reports the configured naming
policy without claiming that PostgreSQL truncates at that smaller limit.

## Options and defaults

`sqlInclude` defaults to `["**/*.sql"]`. `maxBytes` defaults to `63`. `0` and
any value above `63` are configuration errors (`must be from 1 through 63`).
An invalid `sqlInclude` glob is also a configuration error.

## Valid example

A 63-byte name is allowed. A long name that is only referenced is allowed.

```sql
-- db/migrations/001.sql
CREATE TABLE accounts (
  id uuid REFERENCES invoice_line_items_account_id_created_at_status_currency_code(id)
);
CREATE INDEX short_idx ON accounts (id);
```

## Counterexample

```sql
-- db/migrations/001.sql
CREATE INDEX idx_invoice_line_items_account_id_created_at_status_currency_code ON invoice_line_items (account_id);
```

The finding text, after the `{file}:{line}: ` prefix, is:

`identifier "idx_invoice_line_items_account_id_created_at_status_currency_code" is 65 bytes; PostgreSQL truncates identifiers longer than 63 bytes to "idx_invoice_line_items_account_id_created_at_status_currency_co"`

That identifier is 65 bytes. The 63-byte prefix is what PostgreSQL keeps.

## Fix

Shorten the declared name so its UTF-8 length is at most `maxBytes`, and keep
the shortened form unique among names that would otherwise share a prefix.

```sql
CREATE INDEX idx_invoice_lines_account_created_status ON invoice_line_items (account_id);
```

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` (rule option, repository `effort`, or the invocation override) is not an error: the rule has no low implementation, so the check warns `requested effort low; effective effort max because this rule has no low implementation` and runs at `max`. It judges SQL files selected by `sqlInclude`; executor calls in TS/JS are not evaluated.

### Unanalyzable inputs

Valid PostgreSQL can still be unsupported by this analyzer. When it cannot
recover statements such as `ALTER DEFAULT PRIVILEGES`, `CREATE PUBLICATION`,
or `REFRESH MATERIALIZED VIEW`, it diagnoses them under
[`options.unanalyzable`](../migrations/postgres-schema-unanalyzable.md). The
same diagnostic also catches rejected syntax such as `CREATE RECURSIVE VIEW`
without its required explicit column list; it reports missing analyzer facts,
not a PostgreSQL validity verdict.

SQL statements selected by `sqlInclude` that the lenient parser cannot parse or
recover follow `options.unanalyzable`: explicit `report` emits a finding,
`advise` emits a non-failing advisory, and `ignore` omits the diagnostic. The
repository-level default applies when the rule does not override it. Diagnostics
name the source file, statement line, and statement head so the unsupported SQL
can be rewritten or given a reasoned suppression.

Valid sibling statements still contribute schema facts. A real `DISCARD ALL`
statement is parsed normally; the parser's internal recovery marker supplies
no schema proof. File and line suppression directives apply to these diagnostics.
A failure to collect facts still aborts the rule with an error.

For schema parse failures, the implicit default advises for any statement that
cannot be analyzed inside a recognized `DO`, PL/pgSQL function, or procedure body.
This includes malformed body statements: an advisory means analysis is incomplete,
not that PostgreSQL accepts the SQL. Successfully parsed body statements still
contribute facts and rule findings. At the top level, only these complete forms
advise. Malformed and unrecognized statements retain findings.

- `REFRESH MATERIALIZED VIEW [CONCURRENTLY] view [WITH [NO] DATA]`
- `CREATE PUBLICATION name FOR ALL TABLES`
- `CREATE PUBLICATION name FOR TABLE table [, table ...]`
- `ALTER DEFAULT PRIVILEGES [FOR {ROLE | USER} role [, role ...]] [IN SCHEMA schema [, schema ...]] GRANT {ALL [PRIVILEGES] | privilege [, privilege ...]} ON {TABLES | SEQUENCES | FUNCTIONS | ROUTINES | TYPES | SCHEMAS} TO grantee [, grantee ...] [WITH GRANT OPTION]`

Privileges are `SELECT`, `INSERT`, `UPDATE`, `DELETE`, `TRUNCATE`, `REFERENCES`,
`TRIGGER`, `USAGE`, `CREATE`, `EXECUTE`, and `MAINTAIN`. `view` and each `table`
may be schema-qualified; `name`, `role`, `schema`, and `grantee` may not.
`CONCURRENTLY` precedes the view name, and `WITH` requires `DATA`. A matching
prefix with extra tokens is not recognized. Explicit rule or
repository `unanalyzable` policies override these defaults. See the
[schema diagnostic migration](../migrations/postgres-schema-unanalyzable.md).

## Suppression

Put the directive on the statement line, the line before it, or at the top of
the file:

```sql
-- no-mistakes-disable-next-line postgres-identifier-length: audited legacy index
CREATE INDEX legacy_index_name_that_exceeds_the_postgres_limit ON accounts (id);

CREATE INDEX legacy_index_name_that_exceeds_the_postgres_limit ON accounts (id); -- no-mistakes-disable-line postgres-identifier-length: audited

-- no-mistakes-disable-file postgres-identifier-length: legacy migration keeps its names
```

## Related rules

`postgres-require-named-constraints` makes added foreign keys and checks
explicit, which is what gives those constraints a name this rule can measure.
`postgres-object-naming` and `postgres-column-naming` check snapshot names
against project patterns; they see the name PostgreSQL already stored.
