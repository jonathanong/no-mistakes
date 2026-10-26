# `postgres-require-named-constraints`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

Flags unnamed `ALTER TABLE … ADD FOREIGN KEY` and `ADD CHECK` statements.
Named constraints are required so `NOT VALID` adds can pair with
`VALIDATE CONSTRAINT`. Unnamed unique or primary-key adds are not findings.
`CREATE TABLE` inline constraints are not findings.

The rule uses shared schema facts (`extract_migration_facts`,
`collect_postgres_facts`) including `ALTER TABLE` peeled out of `DO $$`
blocks. It does not re-parse SQL with a private parser.

```yaml
rules:
  - rule: postgres-require-named-constraints
    scope: repository
    options:
      sqlInclude: ["db/migrations/**/*.sql"]
```

`sqlInclude` defaults to `**/*.sql`.

Counterexample: an unnamed check or foreign key.

```sql
ALTER TABLE children ADD CHECK (parent_id IS NOT NULL) NOT VALID;
ALTER TABLE children ADD FOREIGN KEY (parent_id) REFERENCES parents(id)
  ON DELETE CASCADE NOT VALID;
```

Fix: give the constraint an explicit name.

```sql
ALTER TABLE children ADD CONSTRAINT children_parent_id_not_null
  CHECK (parent_id IS NOT NULL) NOT VALID;
ALTER TABLE children ADD CONSTRAINT fk_children_parent
  FOREIGN KEY (parent_id) REFERENCES parents(id)
  ON DELETE CASCADE NOT VALID;
```

Use `no-mistakes-disable-next-line postgres-require-named-constraints` or
`no-mistakes-disable-line` for a one-off, or `no-mistakes-disable-file`
when a whole file is an intentional exception.

## Why and when

Named `ADD CONSTRAINT` statements and comment text do not trigger this rule.

Use this rule for migrations that may need phased validation or future schema
maintenance by name.

## What it catches/requires

`ALTER TABLE ... ADD FOREIGN KEY` and `ADD CHECK` statements must name their
constraints. Inline `CREATE TABLE` constraints and unnamed unique/primary-key
adds are intentionally outside the finding set.

## Options and defaults

`sqlInclude` is the only option and defaults to `**/*.sql`. Supported statements
inside `DO $$` blocks are included in the same migration fact pass.

## Valid example

```sql
-- db/migrations/001.sql
ALTER TABLE children ADD CONSTRAINT fk_children_parent
  FOREIGN KEY (parent_id) REFERENCES parents(id);
```

## Counterexample

```sql
-- db/migrations/001.sql
ALTER TABLE children ADD CHECK (parent_id IS NOT NULL);
```

## Fix

Give the check or foreign key a stable name that later migrations can use with
`VALIDATE CONSTRAINT`, replacement, or removal operations.

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

Use `no-mistakes-disable-next-line postgres-require-named-constraints` or the
file directive for generated or externally managed migrations.

## Related rules

[`postgres-constraint-validate`](postgres-constraint-validate.md) pairs named
`NOT VALID` constraints with validation; [`postgres-require-fk-on-delete`](postgres-require-fk-on-delete.md)
requires explicit foreign-key delete semantics.
