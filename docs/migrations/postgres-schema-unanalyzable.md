# PostgreSQL schema statements that cannot be analyzed

The 11 rules below now report each selected SQL statement that the lenient
PostgreSQL parser cannot parse or recover. This changes the default result:
previously such a statement could disappear from schema analysis without a
diagnostic; now `options.unanalyzable: report` produces a finding at its line.
This includes rejected syntax such as `CREATE RECURSIVE VIEW` without its
required explicit column list, which could previously disappear from analysis.

- [`postgres-constraint-validate`](../rules/postgres-constraint-validate.md)
- [`postgres-fk-index`](../rules/postgres-fk-index.md)
- [`postgres-generated-column-predicates`](../rules/postgres-generated-column-predicates.md)
- [`postgres-idempotent-insert`](../rules/postgres-idempotent-insert.md)
- [`postgres-identifier-length`](../rules/postgres-identifier-length.md)
- [`postgres-no-add-column`](../rules/postgres-no-add-column.md)
- [`postgres-no-generated-column-writes`](../rules/postgres-no-generated-column-writes.md)
- [`postgres-redundant-index`](../rules/postgres-redundant-index.md)
- [`postgres-require-fk-on-delete`](../rules/postgres-require-fk-on-delete.md)
- [`postgres-require-named-constraints`](../rules/postgres-require-named-constraints.md)
- [`postgres-sql-statement-policy`](../rules/postgres-sql-statement-policy.md)

A finding means the analyzer could not establish schema facts for that
statement. It does not decide whether PostgreSQL accepts the SQL: valid but
unsupported statements, including `ALTER DEFAULT PRIVILEGES`,
`CREATE PUBLICATION`, and `REFRESH MATERIALIZED VIEW`, can be reported alongside
syntax PostgreSQL rejects. Parsed sibling statements in the same file still
contribute their schema facts. A `COPY ... FROM STDIN` data payload is not
reported as a failed SQL statement. In recognized `DO` blocks and dollar-quoted
PL/pgSQL function and procedure bodies, procedural declarations and assignments,
`IF`/`ELSIF`/`ELSE`,
`CASE`, loops, `EXIT`, `CONTINUE`, `RETURN`, `RAISE`, `PERFORM`, cursor operations,
`SELECT INTO`, `EXECUTE INTO`, diagnostics, and exception handlers are not reported
as failed SQL statements. This does not imply that those commands are executed or
that their conditions are evaluated. Unknown or malformed SQL nested inside their
cursor, loop-query, `SELECT`, or `EXECUTE` payloads still reports, as does unknown
SQL after an `IF ... THEN` or loop prefix. Top-level assignment/declaration-shaped statements,
`END IF`, and unsupported SQL statements after a body still report at their own
lines. Diagnostic heads retain the original statement spelling, including radix
literals such as `0x1F` and `0b101`, through compatibility rewrites. Rejected SQL
actions in procedural branches are reported at the action's line.

The rule's configured roots, path filters, and `sqlInclude` determine which
files are checked. Set `options.unanalyzable` on a rule application to `report`
(the default), `advise` for a non-failing advisory, or `ignore` to omit these
diagnostics. For example:

```yaml
rules:
  - rule: postgres-fk-index
    scope: repository
    options:
      sqlInclude: ["migrations/**/*.sql"]
      unanalyzable: advise
```

Review each new diagnostic before changing policy. Rewrite SQL into supported
static syntax when practical, or suppress one intentional statement at its
line with a reason:

```sql
-- no-mistakes-disable-next-line postgres-fk-index: publication reviewed separately
CREATE PUBLICATION app_changes FOR TABLE accounts;
```

Run a local `no-mistakes check --root . --format json` in your configured project
and validate the changed findings and advisories before adopting the release.

See the [forbidden-calls unknown-call default](forbidden-calls-unknown-calls.md)
for the companion behavior change.
