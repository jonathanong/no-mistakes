# PostgreSQL schema statements that cannot be analyzed

The 11 rules below now report each selected SQL statement that the lenient
PostgreSQL parser cannot parse or recover. This changes the default result:
previously such a statement could disappear from schema analysis without a
diagnostic; now `options.unanalyzable: report` produces a finding at its line.

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

A finding says the analyzer lacks proof, not that PostgreSQL rejects the SQL.
Valid but unsupported statements, including `ALTER DEFAULT PRIVILEGES`,
`CREATE PUBLICATION`, and `REFRESH MATERIALIZED VIEW`, can be reported alongside
invalid SQL. Parsed sibling statements in the same file still contribute their
schema facts. A `COPY ... FROM STDIN` data payload is not reported as a failed
SQL statement. In recognized `DO` blocks and dollar-quoted PL/pgSQL function
and procedure bodies, procedural structure and commands such as `IF`/`ELSIF`/`ELSE`,
`CASE`, loops, `EXIT`, `CONTINUE`, `RETURN`, `RAISE`, and `PERFORM` are not
reported as failed SQL statements. This does not imply that those commands are
executed or that their conditions are evaluated. Unknown SQL nested in a body
still reports, including as the first statement after an `IF ... THEN` or loop
prefix. Top-level `END IF` and unsupported SQL statements
after a body still report at their own lines.

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
