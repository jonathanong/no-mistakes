# PostgreSQL schema statements that cannot be analyzed

<!-- cspell:ignore tablespace OIDS -->

The 11 rules below diagnose each selected SQL statement that the lenient
PostgreSQL parser cannot parse or recover. By default, complete recognized
unsupported PostgreSQL forms produce non-failing advisories. Any statement that
cannot be analyzed inside a recognized `DO`, PL/pgSQL function, or procedure body
also advises by default. Malformed or unrecognized top-level statements still
produce findings at their source lines. Explicit `options.unanalyzable: report`
produces findings for all of these cases.
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
statement. It does not decide whether PostgreSQL accepts the SQL. Valid but
unsupported statements are advised when their complete syntax matches the
conservative recognizer. That recognizer includes:

- `REFRESH MATERIALIZED VIEW [CONCURRENTLY] view [WITH [NO] DATA]`
- `CREATE PUBLICATION name [FOR ALL TABLES | FOR TABLE table [, table ...]] [WITH (option [= value] [, ...])]`, including an empty publication with no `FOR` clause
- `ALTER DEFAULT PRIVILEGES [FOR {ROLE | USER} role [, role ...]] [IN SCHEMA schema [, schema ...]] {GRANT | REVOKE [GRANT OPTION FOR]} {ALL [PRIVILEGES] | privilege [, privilege ...]} ON {TABLES | SEQUENCES | FUNCTIONS | ROUTINES | TYPES | SCHEMAS} {TO | FROM} grantee [, grantee ...]`. `GRANT` may end with `WITH GRANT OPTION`. `REVOKE` may end with `CASCADE` or `RESTRICT`.

Keywords are case-insensitive. A name is a double-quoted identifier or an
unquoted non-keyword. `view` and each `table` may be one schema-qualified name;
`name`, `role`, `schema`, and `grantee` may not. `CONCURRENTLY` comes before
the view name. `WITH` on a refresh requires `DATA`, and `NO` is optional.
Privileges are `SELECT`, `INSERT`, `UPDATE`, `DELETE`, `TRUNCATE`,
`REFERENCES`, `TRIGGER`, `USAGE`, `CREATE`, `EXECUTE`, and `MAINTAIN`. `FOR`
requires `ROLE` or `USER` and comes before `IN SCHEMA`. Publication `WITH`
options are scalar. Publication `WHERE` clauses and column lists are not
recognized.
These forms advise only when neither the repository nor the rule sets
`unanalyzable` or `unanalyzableSql`. An explicit value applies that policy:
`report` makes them findings, `advise` still advises, and `ignore` omits them.
Other rejected top-level forms remain findings: a recognized prefix alone never
proves valid syntax. Recognition checks syntax, not database permissions, object
existence, or privilege/object compatibility. This is not a full PostgreSQL
validator. Parsed sibling statements in the same file still contribute their
schema facts.

The PostgreSQL 17 command-family sweep also recognizes complete selected
metadata forms for access methods, aggregates, casts, conversions, event
triggers, transforms, foreign-data wrappers/tables/servers and user mappings,
foreign-schema imports, ownership changes, security labels, large objects, and
ALTER SYSTEM configuration statements, plus selected CREATE DATABASE options
and ALTER DATABASE SET/RESET forms. Their numeric setting values accept an
optional sign, decimals, and exponents; malformed signs or numeric suffixes
still report.
Selected CREATE/ALTER/DROP forms for groups, languages, routines, publications,
rules, statistics, subscriptions, tablespace objects, and text-search objects follow
the same policy. This covers the saved
[command-family survey](../../fixtures/postgres-facts/source/ddl-docs-survey/families.sql)
and its regression fixtures, rather than every variant of each command's
grammar. Complete metadata recognition produces an advisory without fabricating
schema facts. Missing required clauses and unexpected trailing tokens still
report by default. The grammar reference is the
[PostgreSQL 17 statement list](https://www.postgresql.org/docs/17/sql-commands.html).

Selected PostgreSQL query forms that sqlparser cannot represent also produce
advisories after their full rewritten statement parses: inherited-table `*`,
`COLLATION FOR`, SQL/JSON options and `JSON_TABLE` columns, XML expressions,
interval window bounds, `ROWS FROM`, and `AT LOCAL`. Array-slice INSERT and
UPDATE targets and identity `OVERRIDING` inserts are parsed through a validated
target rewrite so their `ON CONFLICT` and write facts remain available.
Malformed clauses and trailing tokens remain findings.

PostgreSQL 17 `CREATE TABLE` clauses for compression, storage, partition column
lists, generated defaults, and `WITHOUT OIDS` retain typed table facts after
narrow compatibility rewrites. `LIKE ... INCLUDING`, `UNIQUE NULLS NOT
DISTINCT`, and `CHECK ... NOT VALID` use advisories because dropping those
clauses would change constraint facts. Selected table-as-query, base type,
domain, role, schema, policy, index-collation, view-option, and DROP forms use
complete syntax recognition when the parser cannot represent their original
clause. A foreign-key `SET NULL` column list outside its referencing key
remains an advisory; no altered foreign-key fact is inferred.

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
that their conditions are evaluated. Statements recovered from body cursor,
loop-query, `SELECT`, and `EXECUTE`
payloads retain their query, function, and schema facts. Recovery supports
`SELECT ... INTO STRICT`, query loops ending in `FOR NO KEY UPDATE`, and
`FOR ... IN EXECUTE 'literal SQL' LOOP` directly in function bodies as well as
`FOR ... IN EXECUTE format(...) LOOP` with escaped single quotes. Comments accept
newline-continued single-quoted text and dollar-quoted text, including tagged
quotes.

For body `SELECT INTO`, the analyzer validates and retains the full variable
target list, then removes the unsupported `STRICT` modifier in a parser clone.
Targets may include subscripts with nested calls and array literals; commas
inside those expressions do not split the target list.
Unsupported row-lock spellings use the corresponding `UPDATE` or `SHARE` AST
form. Query and function facts remain available; original SQL tokens and source
locations are retained for diagnostics.

If a statement inside a recognized body still cannot be analyzed, the default is
a non-failing advisory, including for unknown or malformed body syntax. This
fallback preserves the diagnostic and its source line without claiming the SQL
is valid. It does not suppress rule findings from successfully recovered sibling
statements. Top-level assignment/declaration-shaped statements, `END IF`, broken
routine wrappers, and malformed SQL before or after a body still produce findings.
An explicit rule or repository policy takes precedence over the body default.

Rust consumers constructing `SqlUnanalyzableStatement` facts must now provide
`procedural_body`: use `false` for top-level statements and `true` for statements
identified inside a routine body. Node options and JSON report shapes are unchanged.

Diagnostic heads retain the original statement spelling, including radix
literals such as `0x1F` and `0b101`, through compatibility rewrites. Rejected
SQL actions in procedural branches retain the action's own line. Invalid labels
before an `IF` remain anchored on that rejected opener. If source views drift,
diagnostics retain their captured tokenized head and location. An explicit
`report` policy reports these actions as findings; the default body policy
advises on them.

The rule's configured roots, path filters, and `sqlInclude` determine which
files are checked. Set `options.unanalyzable` on a rule application to `report`
(explicitly including body statements), `advise` for a non-failing advisory, or
`ignore`
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
