# PostgreSQL fact sources

`no-mistakes` exposes two reusable, rule-free fact sources for PostgreSQL
work. Later check rules should consume these facts instead of re-parsing SQL
or TypeScript.

These extractors are library APIs. There is no CLI command and no check rule
in this change.

## Schema facts

`extract_create_table_metadata(sql)` parses PostgreSQL SQL with the Rust
`sqlparser` crate and returns one `SqlCreateTableMetadata` per `CREATE TABLE`:

- table name (the relation name, not the schema qualifier)
- columns: name, type string, constraint tokens, primary-key flag
- generated-column info (`is_generated`, expression, function name, argument
  columns)

Primary keys are recognized from both `col TYPE PRIMARY KEY` and table-level
`PRIMARY KEY (col)`. Generated columns use `GENERATED ALWAYS AS (...)`
(including `STORED`). Constraint tokens are stable strings such as
`CONSTR_PRIMARY` and `CONSTR_GENERATED`.

Unparseable SQL returns an error. The extractors do not panic.

`extract_schema_facts(root, sources, sql_paths)` reads each path through the
request `SourceStore`. `collect_schema_facts` first filters candidates with
`PostgresSchemaOptions.sql_include` (default `['**/*.sql']`). There is no
hardcoded `backend/migrations/` root.

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

## Out of scope

Lock-ordering and runtime-query *rules* are not part of this fact layer.
Query-name annotations, election-schema vote tables, and UUIDv7 predicates
are also out of scope.
