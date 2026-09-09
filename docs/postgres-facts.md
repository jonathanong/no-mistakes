# PostgreSQL fact sources

`no-mistakes` exposes two reusable, rule-free fact sources for PostgreSQL
work. Check rules consume these facts instead of re-parsing SQL or
TypeScript.

These extractors are library APIs. There is no CLI command or N-API dump.
`postgres-conflict-ordering`, `postgres-lock-ordering`, `postgres-no-offset`,
`postgres-require-query-annotation`,
`postgres-no-generated-column-writes`,
`postgres-fk-index`, `postgres-redundant-index`,
`postgres-constraint-validate`, and `postgres-no-add-column`
consume the facts through `no-mistakes check`. Forthcoming DML rules
(`postgres-required-predicates`, `postgres-sql-shape-policy`,
`postgres-idempotent-insert`) will consume the same INSERT/SELECT facts
once registered.

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

Unparseable statements are skipped. Executable PL/pgSQL `DO`, `CREATE
FUNCTION`, and `CREATE PROCEDURE` bodies are peeled so parseable schema DDL
inside them (`CREATE TABLE`, `CREATE [UNIQUE] INDEX`, `ALTER TABLE`) is
collected, including `ALTER TABLE` after PL/pgSQL `IF/THEN` wrappers. Routine
bodies may be dollar-quoted, plain single-quoted, escaped, Unicode (including a
custom `UESCAPE` character), or newline-concatenated strings. Non-PL/pgSQL
function/procedure bodies remain inert. `chr(n)` calls are rewritten to string
literals, and concatenations of
those literals (`chr(85)||chr(80)||…`) are recovered as SQL when the chunk is
otherwise unparseable. `DROP INDEX CONCURRENTLY` is accepted.
Incomplete statements are still skipped. PostgreSQL 18
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
- `ALTER TABLE … ADD COLUMN`: table, column name, and a source line
- Named `ALTER TABLE … ADD CONSTRAINT … NOT VALID` rows
- `ALTER TABLE … VALIDATE CONSTRAINT` rows

Unparseable statements are skipped, except schema DDL recovered from `DO
$tag$` and PL/pgSQL function/procedure bodies as described above. Statically
recoverable `EXECUTE` literals, qualified or unqualified `format` templates,
and assigned SQL variables contribute every applicable schema-fact family with
lines anchored at the literal/assignment; runtime concatenation remains opaque.
`collect_schema_facts` first filters candidates with
`PostgresSchemaOptions.sql_include` (default `['**/*.sql']`).
There is no hardcoded `backend/migrations/` root.

`postgres-fk-index`, `postgres-redundant-index`, and
`postgres-constraint-validate` consume these
migration facts. `postgres-conflict-ordering`, `postgres-lock-ordering`, and
`postgres-no-generated-column-writes` consume
the facts through `no-mistakes check`.

## Embedded-SQL facts

`extract_embedded_sql_from_source` / `extract_embedded_sql_from_program` walk
an oxc TypeScript AST and resolve the SQL string executed at each database
call site.

Supported argument shapes:

- string literals
- tagged templates, such as a `sql` tag applied to a template literal. The trusted tag is an identifier
  spelled `sql` (case-insensitive) that is not lexically shadowed, or a
  default import from `sql-template-strings` under any local name. Other
  `sql`-named imports, including `import * as sql`, remain untrusted shadows.
  `String.raw` with no interpolations is trusted only when `String` is the
  intrinsic, not a local import, class, parameter, or callable rebinding.
- template literals
- identifiers bound in scope (`const q = \`SELECT ...\`; query(q)`)

Template interpolations become `sql_placeholder_N` (1-based, in source
order). The first quasi is copied as-is; each later quasi is prefixed with
the next placeholder. User-authored text that happens to contain the
`sql_placeholder_` substring is not renumbered when fragments are joined.

### Executor bindings

Imports decide which local identifiers execute SQL:

| knob              | default                  |
| ----------------- | ------------------------ |
| `importSpecifier` | `@data-stores/psql`      |
| `executorNames`   | `query`, `read`, `write` |

Importing `withTransaction` or `withTransactionOptions` also binds `query`.
A missing specifier produces no executor bindings.

A call is a database call when:

- the callee is an identifier in the binding set, or
- the callee is a member expression whose property is `query`

`collect_postgres_facts` runs these extractors when
`CheckFactPlan.postgres_schema`, `CheckFactPlan.embedded_sql`, or
`CheckFactPlan.postgres_dml` is set. `postgres_dml` also extracts statement
facts from matching `.sql` files and from non-`Dynamic` embedded calls.

Each `EmbeddedSqlCall` records `kind`:

- `Inline` — SQL literal or template at the call site
- `ImmutableLocal` — `const` binding with static SQL
- `Composed` — static `+` concatenation, a fluent `.append(...)` chain, or a
  call into a same-file function whose body is exactly one `return` of such a
  chain (recursive, up to 8 calls deep). A parameter referenced outside a
  chain's template-placeholder position, or a callee that isn't a same-file
  function, fails closed instead of resolving.
- `Dynamic` — `let`, reassignment, interpolating templates, or incomplete
  composition (fail closed)

## Statement facts

`extract_sql_statement_facts(sql)` parses PostgreSQL SQL (leniently) and
returns typed INSERT / SELECT / CREATE TRIGGER facts without keeping the
sqlparser AST:

- executed `INSERT` (EXPLAIN without ANALYZE is skipped; PREPARE inner
  statements are treated as executed; CREATE FUNCTION/PROCEDURE bodies are not)
- `ON CONFLICT` action, arbiter (columns / named constraint / unknown), SET
  assignment forms (literal, EXCLUDED, self-ref, COALESCE/GREATEST/LEAST,
  placeholder, volatile, subquery, other), conjunctive WHERE proofs
  (`IS DISTINCT FROM EXCLUDED`, `IS NULL AND EXCLUDED IS NOT NULL`), and
  INSERT column value forms from `VALUES` / `SELECT` (or MySQL-style `SET`).
  Wildcards, omitted columns, `DEFAULT VALUES`, and set operations yield no
  stable form; `DEFAULT`, `CURRENT_TIMESTAMP`, `'now'`, `N'now'`, and `OVERRIDING USER
  VALUE` are unstable
- `INSERT…SELECT` guarded by a conjunctive `WHERE NOT EXISTS`
- SELECT FROM/JOIN relation names, predicate SQL, and `EXISTS` set-operation
  facts (`restricted` when every arm has a placeholder or literal bound;
  `correlated` when a qualified identifier is outside the subquery FROM/WITH)
- `CREATE TRIGGER` table, function, period, row/statement, and events

Unparseable files set `parse_failed` and count quote-masked `INSERT INTO`
keywords so multi-INSERT fragments fail closed. A top-level conjunctive
`WHERE NOT EXISTS` / `AND NOT EXISTS` (paren-depth zero) is recorded even
when the AST is missing.

`postgres-required-predicates`, `postgres-sql-shape-policy`, and
`postgres-idempotent-insert` consume these facts.

## Locking-select facts

`extract_locking_select_metadata(sql)` parses PostgreSQL SQL and returns one
`LockingSelectMetadata` per `SELECT` that uses `FOR UPDATE`:

- `has_multi_row_predicate` — the locked select's `WHERE` uses `IN` or `= ANY`
- `has_order_by` — the locked query has `ORDER BY`
- `skips_locked_rows` — the lock uses `SKIP LOCKED`
- `tables` — the schema-preserving base relations selected by the lock clause
- `table_qualifiers` — the schema, base-name, and alias qualifiers valid for
  each locked relation
- `order` — parsed `ORDER BY` expression keys, used with a configured schema
  catalog to require an exact valid unique-key prefix without accepting a key
  qualified by another joined relation

Unparseable SQL returns an error. The lock-ordering rule consumes this helper
instead of re-parsing SQL with a private parser. `postgres-conflict-ordering`
also consumes request-prepared embedded-SQL facts, then resolves its conflict
arbiter against the configured PostgreSQL schema snapshot. Distinct executor
configurations and catalog paths are prepared once per request and reused by
every rule application that selects them.

`analyze_conflict_inserts(sql)` exposes the same structured SQL projection to
Rust callers as `SqlConflictInsertFact`, `SqlConflictTarget`, and
`SqlInsertSourceShape`. It preserves ordered conflict expressions, partial
predicates, source cardinality, projected target columns, aliases, and parsed
`ORDER BY` keys; rule engines resolve those facts against `SchemaCatalog`
instead of owning another SQL shape.

## Offset facts

`sql_has_offset_clause(sql)` parses PostgreSQL SQL and returns whether any
query uses an `OFFSET` clause, including CTEs, derived tables, subqueries,
`EXISTS`, select-list scalars, `JOIN … ON`, `INSERT`/`UPDATE`/`DELETE`
nested queries, and MySQL `LIMIT offset, limit` form. String literals that
mention the word "offset" are not clauses. Unparseable SQL returns an error.
`postgres-no-offset` consumes this helper.

## Query annotation facts

`sql_requires_query_annotation(sql)` reports whether executed SQL is missing a
leading `/* name */` block comment. `BEGIN` / `COMMIT` / `ROLLBACK` are
exempt, including when they already carry a leading block comment. Line
comments (`-- name`) and empty `/* */` comments are not annotations.
`postgres-require-query-annotation` consumes this helper.

## Out of scope

Lock-ordering and runtime-query _rules_ are not part of this fact layer.
Election-schema vote tables and UUIDv7 predicates are also out of scope.

`postgres-redundant-index` v1 also leaves these migration index transitions
unmodeled: quoted mixed-case identifier quote semantics (`"Events"` versus
`Events`), implicit constraint indexes always recorded at line 1,
`CREATE INDEX IF NOT EXISTS` no-ops, `ALTER INDEX ... RENAME TO`,
`DROP INDEX` / `DROP TABLE` inside `DO $$` blocks, and `ALTER TABLE ...
DROP COLUMN` invalidating indexes.
