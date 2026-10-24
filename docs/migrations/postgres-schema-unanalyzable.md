# PostgreSQL schema statements that cannot be analyzed

The 11 rules below diagnose each selected SQL statement that the lenient
PostgreSQL parser cannot parse or recover. By default, complete recognized
unsupported PostgreSQL forms produce non-failing advisories; malformed or
unrecognized forms produce findings at their source lines. Explicit
`options.unanalyzable: report` produces findings for both classes.
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

A diagnostic means the analyzer could not establish schema facts for that
statement. It does not decide whether PostgreSQL accepts the SQL: valid but
unsupported statements, including `ALTER DEFAULT PRIVILEGES`,
`CREATE PUBLICATION`, and `REFRESH MATERIALIZED VIEW`, are advised when their
complete syntax matches the conservative recognizer.
Recognition covers selected GRANT forms of ALTER DEFAULT PRIVILEGES,
CREATE PUBLICATION with FOR ALL TABLES or plain table lists, and REFRESH
MATERIALIZED VIEW with optional CONCURRENTLY and WITH [NO] DATA.
Other rejected forms remain findings: a recognized prefix alone never proves
valid syntax. Recognition checks syntax, not database permissions, object
existence, or privilege/object compatibility. This is not a full PostgreSQL
validator. Parsed sibling statements in the same file still contribute their
schema facts.

Enum types, triggers, comments on functions/tables/triggers, indexes on `ONLY`
tables, index partition attachments, and text search configurations now parse
without an unanalyzable diagnostic. Where the parser has a matching AST, the
analyzer preserves it. Function signatures in comments and `ONLY` in index
definitions use compatibility normalization. Trigger argument strings are
validated and omitted from the native AST's incompatible parameter-type field;
the trigger's function, target, events, and condition remain available.
Validated utility metadata without
a matching AST uses a neutral analysis marker and contributes no invented schema
facts.

A `COPY ... FROM STDIN` data payload is not
reported as a failed SQL statement. In recognized `DO` blocks and dollar-quoted
PL/pgSQL function and procedure bodies, procedural declarations and assignments,
`IF`/`ELSIF`/`ELSE`,
`CASE`, loops, `EXIT`, `CONTINUE`, `RETURN`, `RAISE`, `PERFORM`, cursor operations,
`SELECT INTO`, `EXECUTE INTO`, diagnostics, and exception handlers are not reported
as failed SQL statements. This does not imply that those commands are executed or
that their conditions are evaluated. Unknown or malformed SQL nested inside their
cursor, loop-query, `SELECT`, or `EXECUTE` payloads still reports, as does unknown
SQL after an `IF ... THEN` or loop prefix. Top-level assignment/declaration-shaped statements,
`END IF`, and unsupported SQL statements after a body retain diagnostics at their own
lines, using the same default classification.

The rule's configured roots, path filters, and `sqlInclude` determine which
files are checked. Set `options.unanalyzable` on a rule application to `report`
(explicitly for both classes), `advise` for a non-failing advisory, or `ignore`
to omit these diagnostics. For example:

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
