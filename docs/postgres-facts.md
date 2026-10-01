# PostgreSQL fact sources

`no-mistakes` exposes two reusable, rule-free fact sources for PostgreSQL
work. Check rules consume these facts instead of re-parsing SQL or
TypeScript.

These extractors are library APIs. There is no CLI command or N-API dump.
`postgres-conflict-ordering`, `postgres-lock-ordering`,
`postgres-column-requires-trigger`, `postgres-required-comments`,
`postgres-duplicate-function-body`, `postgres-table-shape`,
`postgres-status-with-lifecycle-timestamps`,
`postgres-no-offset`,
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
for the parser). Column lists on `ON DELETE SET NULL` and
`ON DELETE SET DEFAULT` are accepted; the recorded action omits the
column list (`SET NULL`, `SET DEFAULT`) and the constraint name is unchanged.
`ON UPDATE` column lists are not rewritten, because PostgreSQL accepts a
column subset only for `ON DELETE`.
A file that cannot be tokenized yields no tables. The
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
  referenced table, optional `ON DELETE` action (column lists on
  `ON DELETE SET NULL` / `SET DEFAULT` are omitted from the action string),
  and a source line
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
This is the lock-ordering `sqlText` contract. It is intentionally different
from Filaments' runtime-query helper, which joins quasis with `?`.

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

The extractor also records recoverable SQL fragments returned from builders or
passed to `.append(...)`. Structural policies may inspect those fragments
without treating the builder as executed DML. Builder recovery uses the same
trusted-tag and lexical-shadow rules as executed SQL. A raw appended
identifier that cannot be resolved becomes a synthetic qualified outer
reference, so shape policies conservatively retain possible correlation while
runtime table constants and column fragments remain statically parseable.
Fragments that exactly match an executed call are inspected once.

Each `EmbeddedSqlCall` records `kind`:

- `Inline` — SQL literal or template at the call site
- `ImmutableLocal` — `const` binding with static SQL
- `Composed` — static `+` concatenation, a fluent `.append(...)` chain, a
  statement-level `.append(...)` mutation on a bound `SQLStatement` (including
  `sql-template-strings` init plus later `query.append(...)` in the same
  function), or a call into a same-file function whose body is a single
  `return` of such a chain or a straight-line `const`/`let` plus `.append`
  mutations ending in `return` (recursive, up to 8 calls deep). Conditional
  static appends keep the recovered base SQL and classify `Dynamic` so an
  INSERT cannot pass a branch-only `ORDER BY` as if it always ran; recovered
  non-INSERT stays ignored by conflict-ordering. Opaque statement-level and
  fluent appends likewise keep a verified leading `SELECT`, `UPDATE`, `INSERT`,
  `DELETE`, or `MERGE` as partial text. Incomplete prefixes such as `WITH`
  remain fully opaque. Loops, nested functions that mutate an outer binding,
  control flow in a helper, a parameter referenced outside a chain's
  template-placeholder position, or a callee that isn't a same-file function,
  classify as dynamic.
- `Dynamic` — `let`, reassignment, interpolating templates, or incomplete
  composition. A present `sql_text` can be verified leading text rather than
  the complete runtime statement; consumers must use it only for conservative
  statement classification.

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
every rule application that selects them. `postgres-column-requires-trigger`
reads column names and trigger definitions from the same schema catalog.
Trigger matching compares the unqualified function, timing, event subset, and
row-ness. Snapshot findings use `table:<name>` object refs and `allow`
entries. `postgres-required-comments` reads table, column, and view comments
from that catalog. `table` includes partitioned tables. An empty
`columnNamePatterns` list checks every column, then
`exemptColumnNamePatterns` skips matches.
`postgres-duplicate-function-body` reads function bodies and languages from
that catalog, tokenizes each body with the PostgreSQL lexer, and groups
functions whose normalized tokens match. `postgres-table-shape` reads
tables, columns, foreign keys, primary keys, triggers, and enum names from
that catalog and checks them against configured name patterns.
`postgres-status-with-lifecycle-timestamps` reads column names, data types,
and generated flags from that catalog.

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

## Schema catalog model

`SchemaCatalog::load` reads a repository-relative PostgreSQL snapshot with
`formatVersion: 2`. Missing snapshot fields default, so a catalog that only
records indexes still loads. `postgres-conflict-ordering` and
`postgres-lock-ordering` keep resolving arbiters from valid, ready, unique or
primary btree indexes only. `tables()`, `table(name)`, `functions()`,
`enums()`, and `views()` expose the rest of the snapshot. Iterators are in
name order. Function order follows the snapshot key. `table(name)` matches
that key exactly.

`SCHEMA_CATALOG_RULE_IDS` selects which configured rules load a snapshot.
It is a separate list from `PREPARED_EMBEDDED_SQL_RULE_IDS`, so a schema-only
rule can append its id without declaring an embedded-SQL executor.

JSON keys are camelCase. A table object may contain `relationKind`
(`table`, the default, or `partitioned table`), `comment`, `columns`,
`primaryKey` (`null` means no primary key), `foreignKeys`,
`checkConstraints`, `uniqueConstraints`, `indexes` (every index, not only
unique btree indexes), `triggers`, and `physicalPartition` (`null` means the
table is not partitioned). Column objects carry `dataType` (verbatim, such
as `timestamp with time zone` or `text[]`), `nullable`, `defaultExpression`,
`generated` (`stored` or `virtual`), `generatedExpression`, `identity`,
`comment`, and `ordinalPosition`. Columns are sorted by `ordinalPosition`,
then name. Foreign-key `onDelete` and `onUpdate` are stored lowercased.
`physicalPartition.key` is text such as `RANGE (id)`, `LIST (region)`,
`HASH (id)`, or `RANGE (tenant_id, id)`. The first word is the strategy.
Top-level commas split the parenthesized elements. A bare or double-quoted
identifier is a column. Anything else is an expression. A comma that sits
inside a function call stays part of that expression.
An unknown strategy fails the load with
`schemaCatalogPath <path>: table <t> has unsupported partition strategy <s>`.

Trigger objects carry `definition`. The definition is parsed with a small
tokenizer, including the legacy `EXECUTE PROCEDURE` spelling. A `CONSTRAINT`
keyword may sit between `CREATE` and `TRIGGER`. Clauses that name a referenced
table, say whether the trigger is deferrable, or introduce transition tables
are skipped. Events are separated by `OR`. `WHEN (...)` keeps its text without
the outer parentheses.
Arguments are single-quoted SQL literals (`''` is an escaped quote, and a
comma inside quotes does not split). The function name drops its schema
qualifier. An unquoted name folds to lowercase (`public.Fn_Touch` becomes
`fn_touch`). A quoted name keeps its case and loses the quotes
(`public."Touch"` becomes `Touch`).
`CatalogTrigger::matches(function, timing, events, for_each_row)` is true
when that stored function, timing, and row-ness are equal and `events`
is a subset of the trigger's events.

A function object carries `definition`. `name` is the snapshot key up to the
first `(`, and `signature` is the text inside those parentheses. `language`
is the word after `LANGUAGE`, lowercased. `returns_trigger` is true when the
definition, before its body, contains `RETURNS trigger` as a word, ignoring
comments and quoted text. An event trigger, or a `RETURNS` clause that yields
a set of rows, does not set it. An event trigger is recorded on its own.
`body` is the SQL inside a
dollar quote (including an empty `$$` tag), a plain or escape string after
`AS`, a `BEGIN ATOMIC` block, or a `RETURN` expression, and the body's byte
range is kept with it. Both are absent when no body is found. Null-input
behavior, security, parallel mode, leakproof, and volatility are stored on
their own. Those mode words count only after the argument list. The return
contract keeps the `RETURNS` clause, including a `RETURNS TABLE` column list,
`OUT` and `INOUT` parameters, and Unicode letters in type names. `COST`,
`ROWS`, and `SUPPORT` are stored separately from the body. Enum objects
carry `values`. View objects carry `materialized`, `definition`, and
`comment`.

Snapshot findings use an object ref as their stable id. `Display` and
`FromStr` share this syntax. An unknown prefix or an empty part is an error.

- `table:<t>`
- `column:<t>.<c>`
- `index:<t>.<i>`
- `trigger:<t>.<trg>`
- `constraint:<t>.<name>`
- `function:<snapshot key>`
- `enum:<e>`
- `view:<v>`
- `materialized-view:<v>`

For a column, index, trigger, or constraint, the last `.` separates the
snapshot table key from the object name. `column:public.orders.id` is table
`public.orders` and column `id`.

`catalog_finding(rule_id, catalog_path, object, text)` sets `file` to the
slash-normalized catalog path, `line` to 1, `target` to the object ref, and
`message` to `{catalog_path}: {object}: {text}`. Line 1 is deliberate: a
snapshot has no source comment to attach a suppression to. The object ref is
the locator.

`AllowList::compile(rule_id, entries)` reads `{object, reason}` entries. An
empty or whitespace `reason` fails with
`<rule-id> option allow: entry <object> needs a reason`. An unparseable
object fails with `<rule-id> option allow: invalid object ref <object>`. A
duplicate object fails with `<rule-id> option allow: duplicate entry <object>`.
`AllowList::apply` drops findings whose `target` equals an entry and adds one
finding per unused entry:
`{catalog_path}: stale <rule-id> allow entry: <object>`, again at line 1.
`require_catalog_path(rule_id, path)` fails with
`<rule-id> option schemaCatalogPath: required` when the path is empty.

## Out of scope

Lock-ordering and runtime-query _rules_ are not part of this fact layer.
Election-schema vote tables and UUIDv7 predicates are also out of scope.

`postgres-redundant-index` v1 also leaves these migration index transitions
unmodeled: quoted mixed-case identifier quote semantics (`"Events"` versus
`Events`), implicit constraint indexes always recorded at line 1,
`CREATE INDEX IF NOT EXISTS` no-ops, `ALTER INDEX ... RENAME TO`,
`DROP INDEX` / `DROP TABLE` inside `DO $$` blocks, and `ALTER TABLE ...
DROP COLUMN` invalidating indexes.
