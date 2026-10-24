# `postgres-no-add-column`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

Flags `ALTER TABLE … ADD COLUMN` in PostgreSQL schema SQL. New columns belong
in the original `CREATE TABLE` so deployed databases are not widened in
place. `ADD CONSTRAINT` and `CREATE TABLE` are not findings.

The rule uses shared schema facts (`extract_migration_facts`,
`collect_postgres_facts`) including `ALTER TABLE` peeled out of `DO $$`
blocks. It does not re-parse SQL with a private parser.

```yaml
rules:
  - rule: postgres-no-add-column
    scope: repository
    options:
      sqlInclude: ["db/migrations/**/*.sql"]
```

When a deployed schema needs a deliberately narrow exception, `allowedMigrations`
declares the complete parsed operation. Every configured entry must match one
analyzed `ALTER TABLE … ADD COLUMN`, and every analyzed add-column must match a
configured entry while the list is present. This makes unexpected migrations and
stale exceptions both fail.

The following block is an allowlist reference. Valid and Counterexample execute
the `sqlInclude` glob in the first example.

```yaml
rules:
  - rule: postgres-no-add-column
    scope: repository
    options:
      sqlInclude: ["backend/data-stores/psql/migrations/**/*.sql"]
      allowedMigrations:
        - path: backend/data-stores/psql/migrations/001-add-status.sql
          table: posts
          column: status
          type: TEXT
          nullable: false
          default: "'draft'"
```

`path`, schema-qualified `table`, `column`, `type`, `nullable`, and `default`
compare exactly against the analyzer's canonical PostgreSQL parser output. Each
entry permits one matching operation; repeated identical `ADD COLUMN`
statements require distinct entries, and duplicate entries are rejected. For a
column with no default, omit `default`. Direct schema statements in executable
PL/pgSQL `DO`, function, and procedure bodies are analyzed like other migration
SQL, including dollar, plain, escaped, Unicode (with custom `UESCAPE`), and
newline-concatenated body strings. Statically recoverable `EXECUTE` strings,
qualified or unqualified `format()` templates, and variables assigned one of
those expressions are also analyzed. Runtime-built expressions remain
intentionally opaque and cannot satisfy an allowlist entry.

`sqlInclude` defaults to `**/*.sql`.

Counterexample: a later migration adds a column.

```sql
ALTER TABLE posts ADD COLUMN status text;
```

Fix: fold the column into the original `CREATE TABLE`.

```sql
CREATE TABLE posts (
  id uuid PRIMARY KEY,
  status text
);
```

Use `no-mistakes-disable-next-line postgres-no-add-column` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.

## Why and when

Use this rule when schema shape must be established in the original table
definition rather than widened by a later migration.

## What it catches/requires

Every included `ALTER TABLE ... ADD COLUMN` must either be absent or match one
complete `allowedMigrations` entry. Stale allowlist entries also fail.

## Options and defaults

`sqlInclude` defaults to `**/*.sql`; `allowedMigrations` defaults to an empty
list, meaning no add-column operation is allowed. Entries compare `path`,
`table`, `column`, `type`, `nullable`, and optional `default` exactly.

## Valid example

```sql
-- db/migrations/001.sql
CREATE TABLE posts (
  id uuid PRIMARY KEY,
  status text NOT NULL DEFAULT 'draft'
);
```

## Counterexample

```sql
-- db/migrations/001.sql
ALTER TABLE posts ADD COLUMN status text;
```

## Fix

Move the column into the original `CREATE TABLE`, or add a fully specified
allowlist entry for a deliberate deployed-schema migration.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` (rule option, repository `effort`, or the invocation override) is not an error: the rule has no low implementation, so the check warns `requested effort low; effective effort max because this rule has no low implementation` and runs at `max`. It judges SQL files selected by `sqlInclude`; executor calls in TS/JS are not evaluated.

### Unanalyzable inputs

Valid PostgreSQL can still be unsupported by this analyzer. When it cannot
recover statements such as `ALTER DEFAULT PRIVILEGES`, `CREATE PUBLICATION`,
or `REFRESH MATERIALIZED VIEW`, it reports them under
[`options.unanalyzable`](../migrations/postgres-schema-unanalyzable.md). The
same diagnostic also catches rejected syntax such as `CREATE RECURSIVE VIEW`
without its required explicit column list; it reports missing analyzer facts,
not a PostgreSQL validity verdict.

SQL statements selected by `sqlInclude` that the lenient parser cannot parse or
recover follow `options.unanalyzable`: `report` (default) emits a finding,
`advise` emits a non-failing advisory, and `ignore` omits the diagnostic. The
repository-level default applies when the rule does not override it. Diagnostics
name the source file, statement line, and statement head so the unsupported SQL
can be rewritten or given a reasoned suppression.

Valid sibling statements still contribute schema facts. A real `DISCARD ALL`
statement is parsed normally; the parser's internal recovery marker supplies
no schema proof. File and line suppression directives apply to these diagnostics.
A failure to collect facts still aborts the rule with an error.

## Suppression

Use `no-mistakes-disable-next-line postgres-no-add-column` for a one-off
exception, or the file directive for a migration family with an external
approval process.

## Related rules

[`postgres-sql-statement-policy`](postgres-sql-statement-policy.md) restricts
statement kinds in non-migration SQL; [`postgres-constraint-validate`](postgres-constraint-validate.md)
checks phased constraint validation.
