# PostgreSQL fact sources

`no-mistakes` exposes two reusable, rule-free fact sources for PostgreSQL
work. Check rules consume these facts instead of re-parsing SQL or
TypeScript.

These extractors are library APIs. There is no CLI command or N-API dump.
`postgres-lock-ordering`, `postgres-no-offset`,
`postgres-no-generated-column-writes`,
`postgres-fk-index`, `postgres-redundant-index`, and
`postgres-constraint-validate` consume
the facts through `no-mistakes check`.

## Schema facts

`extract_create_table_metadata(sql)` parses PostgreSQL SQL with the Rust
`sqlparser` crate and returns one `SqlCreateTableMetadata` per parseable
`CREATE TABLE`:

- table name (the relation name, not the schema qualifier)
- columns: name, type string, constraint tokens, primary-key flag
- generated-column info (`is_generated`, expression, function name, argument
  columns)

Primary keys are recognized from both `col TYPE PRIMARY KEY` and table-level
`PRIMARY KEY (col)`. Generated columns use `GENERATED ALWAYS AS (...)`
(including `STORED` and PostgreSQL 18 `VIRTUAL`). Constraint tokens are
stable strings such as `CONSTR_PRIMARY` and `CONSTR_GENERATED`.

Unparseable statements are skipped. `DO $tag$` bodies are peeled so
parseable schema DDL inside them (`CREATE TABLE`, `CREATE [UNIQUE] INDEX`,
`ALTER TABLE`) is collected, including `ALTER TABLE` after PL/pgSQL
`IF/THEN` wrappers. `CREATE FUNCTION` / `CREATE PROCEDURE` bodies are not
peeled. Other sqlparser-rejected SQL (`chr()`-built fragments, incomplete
statements) is still skipped. PostgreSQL 18
`GENERATED ALWAYS AS (...) VIRTUAL` is accepted (rewritten to `STORED`
for the parser). A file that cannot be tokenized yields no tables. The
extractors do not panic.

`extract_schema_facts(root, sources, sql_paths)` reads each path through the
request `SourceStore` and runs `extract_migration_facts`, which includes
`CREATE TABLE` plus:

- `CREATE INDEX` / unique and primary-key covering indexes: table, optional
  name (schema-qualified when written that way), key columns (name, opclass,
  ordering, nulls), `INCLUDE` columns, uniqueness, access method (`USING`
  defaults to btree), whether a `WHERE` predicate is present, a predicate key
  that lowercases keywords and unquoted identifiers but keeps string literals and
  quoted identifiers, a `col IS NOT NULL` predicate column when that is the
  whole predicate, and a source line taken from that statement's occurrence in
  the file (so a wrapped `CREATE INDEX` still points at the `CREATE` line)
- `DROP INDEX` names (schema-qualified) and source lines, so later drops can
  remove earlier creates of the same identity
- `DROP TABLE` names and source lines, so later table drops can remove that
  table's indexes
- Foreign keys from `CREATE TABLE` and `ALTER TABLE`: table, columns,
  referenced table, optional `ON DELETE` action, and a source line
- Named `ALTER TABLE … ADD CONSTRAINT … NOT VALID` rows
- `ALTER TABLE … VALIDATE CONSTRAINT` rows

Unparseable statements are skipped, except schema DDL recovered from `DO
$tag$` bodies as described above. `collect_schema_facts` first filters
candidates with `PostgresSchemaOptions.sql_include` (default `['**/*.sql']`).
There is no hardcoded `backend/migrations/` root.

`postgres-fk-index`, `postgres-redundant-index`, and
`postgres-constraint-validate` consume these
migration facts. `postgres-lock-ordering` and
`postgres-no-generated-column-writes` consume
the facts through `no-mistakes check`.

## Embedded-SQL facts

`extract_embedded_sql_from_source` / `extract_embedded_sql_from_program` walk
an oxc TypeScript AST and resolve the SQL string executed at each database
call site.

Supported argument shapes:

- string literals
- tagged templates (`sql\`SELECT ...\``)
- template literals
- identifiers bound in scope (`const q = \`SELECT ...\`; query(q)`)

Template interpolations become `sql_placeholder_N` (1-based, in source
order). The first quasi is copied as-is; each later quasi is prefixed with
the next placeholder. This is the lock-ordering `sqlText` contract. It is
intentionally different from Filaments' runtime-query helper, which joins
quasis with ` ? `.

### Executor bindings

Imports decide which local identifiers execute SQL:

| knob | default |
| --- | --- |
| `importSpecifier` | `@data-stores/psql` |
| `executorNames` | `query`, `read`, `write` |

Importing `withTransaction` or `withTransactionOptions` also binds `query`.
A missing specifier produces no executor bindings.

A call is a database call when:

- the callee is an identifier in the binding set, or
- the callee is a member expression whose property is `query`

`collect_postgres_facts` runs these extractors only when
`CheckFactPlan.postgres_schema` or `CheckFactPlan.embedded_sql` is set.

`postgres-no-generated-column-writes` consumes these facts. It demands both
`postgres_schema` and `embedded_sql`, then matches parsed DML writes against
generated columns. Tables that are not declared in SQL go through that
rule's `extraGeneratedColumns` option; this layer does not scrape
`voteTable:` literals.

## Locking-select facts

`extract_locking_select_metadata(sql)` parses PostgreSQL SQL and returns one
`LockingSelectMetadata` per `SELECT` that uses `FOR UPDATE`:

- `has_multi_row_predicate` — the locked select's `WHERE` uses `IN` or `= ANY`
- `has_order_by` — the locked query has `ORDER BY`
- `skips_locked_rows` — the lock uses `SKIP LOCKED`

Unparseable SQL returns an error. The lock-ordering rule consumes this helper
instead of re-parsing SQL with a private parser.

## Offset facts

`sql_has_offset_clause(sql)` parses PostgreSQL SQL and returns whether any
query uses an `OFFSET` clause, including CTEs, derived tables, subqueries,
`EXISTS`, select-list scalars, `JOIN … ON`, `INSERT`/`UPDATE`/`DELETE`
nested queries, and MySQL `LIMIT offset, limit` form. String literals that
mention the word "offset" are not clauses. Unparseable SQL returns an error.
`postgres-no-offset` consumes this helper.

## Out of scope

Lock-ordering and runtime-query *rules* are not part of this fact layer.
Query-name annotations, election-schema vote tables, and UUIDv7 predicates
are also out of scope.

`postgres-redundant-index` v1 also leaves these migration index transitions
unmodeled: quoted mixed-case identifier quote semantics (`"Events"` versus
`Events`), implicit constraint indexes always recorded at line 1,
`CREATE INDEX IF NOT EXISTS` no-ops, `ALTER INDEX ... RENAME TO`,
`DROP INDEX` / `DROP TABLE` inside `DO $$` blocks, and `ALTER TABLE ...
DROP COLUMN` invalidating indexes.
