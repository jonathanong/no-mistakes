<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->
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
`postgres-constraint-validate`, `postgres-no-add-column`,
`postgres-column-naming`, `postgres-finite-text-columns`,
`postgres-array-columns`, and
`postgres-identifier-length`, `postgres-required-predicates`, and
`postgres-sql-shape-policy`
consume the facts through `no-mistakes check`. Forthcoming DML rules
(`postgres-idempotent-insert`) will consume the same INSERT/SELECT facts
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
- `ALTER TABLE … ADD COLUMN`: table, column name, type, nullability, default,
  `is_generated` status, and a source line. `unqualified_table_name` preserves
  the decoded last identifier component, including dots inside quoted names.
- `table_events`: ordered `SqlTableSchemaEvent::Create`, `AddColumn`, and
  `Drop` operations. Each records the qualified `table` key and decoded
  `unqualified_table` component; CREATE and ADD retain column metadata.
  Unquoted event identifiers fold to lowercase, quoted names preserve case,
  and table-level primary keys remain attached to their columns. Lexical
  `source_order` ordinals interleave direct and statically recovered routine
  DDL, including multiple operations on the same source line. Live catalogs
  apply CREATE as replacement, ADD in positional order, and DROP as removal.
- Declared identifiers: each name a statement introduces, unquoted and without
  a schema qualifier, plus that statement's line. This covers `CREATE TABLE`
  (the table, its columns, and inline or table `CONSTRAINT` names),
  `CREATE [UNIQUE] INDEX`, `CREATE TRIGGER`, `CREATE [OR REPLACE] FUNCTION`,
  `CREATE PROCEDURE`, `CREATE [MATERIALIZED] VIEW`, `CREATE TYPE`, and
  `ALTER TABLE` / `ALTER INDEX` names introduced by `ADD CONSTRAINT`,
  `ADD COLUMN`, `RENAME TO`, `RENAME COLUMN … TO`, and `RENAME CONSTRAINT … TO`.
  Referenced names, including `REFERENCES` targets, are omitted. Dollar-quoted
  `DO` bodies are included in this list; routine bodies are merged with lines
  remapped onto the outer file. `postgres-identifier-length` reads this list
  and does not parse SQL itself. `SqlDeclaredIdentifier` is re-exported from
  `codebase::postgres` for Rust callers. A single token index accounts for
  declaration modifiers, nested comments, Unicode procedure names, and explicit
  view-column declarations; routine bodies have their own location context.
  Schema-rule projections borrow these facts; enabling additional migration rules
  does not clone their nested metadata. Standalone filesystem dispatch declares
  schema, catalog, and embedded SQL demand before its shared fact pass.
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

Configured migration rules declare the union of `sqlInclude` patterns before
request collection. Identifier length, added-column, named-constraint, index,
foreign-key, validation, and statement-policy checks borrow the request's
prepared schema projection, including I/O failures. Selecting a narrower rule
scope filters that projection without another migration parse.

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

| knob | default |
| --- | --- |
| `importSpecifier` | Empty; configure your database module explicitly |
| `executorNames` | Empty without a module; `query`, `read`, `write` with a module |

Without a module, only explicitly configured names match named imports from
any module. With neither option set, no executor calls are collected.
A configured module's `withTransaction` and `withTransactionOptions` imports
also bind `query`. Member calls named `query` are collected when a module is configured or `query`
is explicitly enabled in the executor names. Existing SQL-builder fragment recovery is
independent of executor selection.

See [the breaking-change migration](migrations/explicit-postgres-executors.md).

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
  `correlated` when a qualified identifier is outside the subquery FROM/WITH;
  `column` is the `EXISTS` keyword in the original SQL)
- one `SqlRelationPredicateFact` per base-table instance of each SELECT
  (`relations`), plus grouped `updates` and `deletes` (one group per
  statement). Each fact records the table, optional alias, source line,
  `constrained_columns` (qualified, or unqualified when that statement has a
  single FROM item), and `unqualified_columns` (bare names from a
  multi-relation FROM that a schema catalog may still prove). A column is
  constrained by a top-level AND conjunct that is an equality, comparison,
  `IN`, `= ANY`, or `BETWEEN`; `OR` keeps a column only when every branch
  constrains it. `in_insert_select` marks SELECT facts that belong to
  `INSERT … SELECT`, including selects nested in that query.
  `not_in_subqueries` records lines of `NOT IN (SELECT …)` and
  `NOT (… IN (SELECT …))`. `count_existence_checks` records each `COUNT(*)`
  compared with 0 or 1 as a `SqlCountExistenceFact`: the comparison's line,
  its SQL column, and `negated` (true when the comparison tests zero rows).
  `postgres-sql-shape-policy` reads both when those shapes are banned.
  `star_projections` records each bare `*` (one fact per base FROM relation) and
  each `alias.*` (that alias's base relation). Stars inside `EXISTS` / `NOT EXISTS`,
  `COUNT(*)`, CTEs, and derived-table aliases are omitted. A star passed to a
  function records `within_function` so `postgres-explicit-columns` can apply
  `allowWholeRowFunctions` without parsing SQL again.
  `returning_stars` on the file facts records `RETURNING *` / `RETURNING t.*`
  for INSERT, UPDATE, and DELETE.
  `column_uses` records bare column references from comparisons in WHERE and
  JOIN ON, and from ORDER BY. An empty `table` means the name was unqualified
  across more than one base relation. `postgres-generated-column-predicates`
  reads these facts together with migration schema facts.
- `CREATE TRIGGER` table, function, period, row/statement, and events

Unparseable files set `parse_failed` and count quote-masked `INSERT INTO`
keywords so multi-INSERT fragments fail closed. A top-level conjunctive
`WHERE NOT EXISTS` / `AND NOT EXISTS` (paren-depth zero) is recorded even
when the AST is missing.

`postgres-required-predicates` consumes the relation-predicate facts and, when
`partitionKeys` is `require`, the schema catalog's relation kind, columns, and
partition key. `postgres-sql-shape-policy` and `postgres-idempotent-insert`
consume the same statement facts.

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
and generated flags from that catalog. `postgres-object-naming` reads table,
index, trigger, function, view, and enum names from that catalog.
`postgres-column-naming` reads columns and foreign keys from that catalog.

`analyze_conflict_inserts(sql)` exposes the same structured SQL projection to
Rust callers as `SqlConflictInsertFact`, `SqlConflictTarget`, and
`SqlInsertSourceShape`. It preserves ordered conflict expressions, partial
predicates, source cardinality, projected target columns, aliases, and parsed
`ORDER BY` keys; rule engines resolve those facts against `SchemaCatalog`
instead of owning another SQL shape.

## Offset facts

`sql_offset_uses(sql)` parses PostgreSQL SQL and returns each `OFFSET` clause
as `OffsetUse::Zero` (the integer literal `0`, including `OFFSET 0 ROWS`) or
`OffsetUse::Other`. `sql_has_offset_clause(sql)` is that list's emptiness.
The walk includes CTEs, derived tables, subqueries, `EXISTS`, select-list
scalars, `JOIN … ON`, `INSERT`/`UPDATE`/`DELETE` nested queries, and MySQL
`LIMIT offset, limit` form. String literals that mention the word "offset"
are not clauses. Unparseable SQL returns an error. `sql_file_offset_uses`
uses the statement pass's lenient split and reports each use on a top-level
query or `CREATE [MATERIALIZED] VIEW` at that statement's line.
`postgres-no-offset` consumes these helpers.

## Row-bound facts

For `UPDATE` and `DELETE`, a bound fact's location is the statement keyword,
while its relation items retain their own locations. Embedded SQL maps both to
physical source lines so a suppression on the statement start covers its findings.

`SqlStatementFileFacts.bounds` holds one `SqlBoundFact` per executed `SELECT`,
`UPDATE` and `DELETE`, including those in data-modifying CTEs (whose `RETURNING` rows are an
`Opaque` item of the query that reads them) and under `EXPLAIN ANALYZE`; `INSERT … SELECT`
is not a bound fact. A fact keeps the `kind`, its
`line` and `column`, a `SqlBoundQuery`, and for `UPDATE` / `DELETE` the index of the
`target` item. Every item keeps its own `line` and `column`, so SQL recovered from
several source operands maps each relation to the operand that wrote it.
A `SqlBoundQuery` is `capped` when it has a `LIMIT` / `FETCH FIRST n ROWS ONLY` with a
fixed count (a literal, a bind or an expression of them; `LIMIT NULL`, `LIMIT ALL`, a count
taken from a subquery or a column, `FETCH … WITH TIES` and `FETCH … PERCENT` do not cap) or
is a pure aggregate (a built-in aggregate call, bare or `pg_catalog.`-qualified, not
windowed, in the select list or `HAVING`, with no `GROUP BY` and no set-returning function
in the select list), and lists its FROM `items`. An item is a base `Table`, a `Query`
(a CTE reference carrying that CTE's own query, a derived table, or one arm of a set
operation), `Other` (a `VALUES` list, or a set-returning built-in such as `unnest($1)`
over arguments the statement supplies), or `Opaque` (any other table function, and the
recursive reference of a recursive CTE: never reported, and it bounds nothing pinned to
it). A CTE reference without an alias is addressed by the CTE's name. A `COPY (SELECT …)`
query is a `Select` fact.

Each item retains positional `column_aliases` as syntax, so catalog consumers can avoid confusing renamed columns with base names.

Each item lists the `pins` that top-level `AND` conjuncts impose on its columns: an
equality or `IS NOT DISTINCT FROM` (a `null_safe` pin), `= ANY(…)`, `IN (…)` or
`IN (SELECT …)` against a
`Value` (no relation of the statement: a literal, a bind, or an interpolation recovered
from a template literal), `Items` (columns of other items; a column
compared with its own item is never a pin), `Array` (finite constructor dependencies plus scalar column requirements), or a subquery. An `Array` source keeps source-item indexes and column names; the evaluator proves those columns scalar against the prepared catalog before crediting their row bounds. `cast_types` records custom cast targets that require catalog scalar or enum proof; interval literals are already scalar. `indexed_columns` records columns accessed exclusively by scalar subscripts: the evaluator requires a catalog array with a proven scalar element type. Slices, unknown/domain element types, and unresolved aliases supply no indexed proof. Array columns without a scalar subscript supply no scalar proof. `WHERE` restricts every item.
A join condition restricts only the non-preserved side of an outer join and both sides of
an inner join; `USING (col)` pins like `ON a.col = b.col` when each side is one item,
and `FULL`, `NATURAL` and `CROSS` joins pin nothing. A subquery that reads the row being
checked, a value with an explicit `COLLATE`, or a call to a built-in function that differs per
row, is not a pin source. Which relation owns a bare column of a subquery is known only to
the catalog, so a pin and a `LATERAL` item keep `reads` (`lateral_reads`): each bare column
with the base tables that could own it, outward when none of them has the column.
A DML target is always a `Table`, never a CTE reference, and a `TABLE name` set-operation
arm is a `Table` item, or the CTE it names. A bare column
among several items has no provable owner and an unknown qualifier is an outer
reference, so neither pins. Only base tables take pins. The facts are syntactic: a rule
decides against a catalog whether a pinned column set is a unique key.

`SqlStatementFileFacts.limit_uses` lists every `LIMIT` / `FETCH FIRST` count in the
executed statements (CTEs and subqueries included) at its line and column, as
`Literal(n)` for an integer literal (`FETCH FIRST ROW ONLY` is `Literal(1)`, located at
its `FETCH` keyword) or `Other`
for a placeholder, expression or percentage. `LIMIT NULL` and `LIMIT ALL` are not
recorded. `sweeps` lists each limited query (one that caps its rows: not `WITH TIES`, not
`LIMIT 0`) over one base table (no join, grouping or set operation; parentheses around
the `SELECT` are looked through) ordered only by plain columns of that table (a bare
`ORDER BY` name that is an output alias means the aliased expression), at the line and
`column` of the table, with its top-level `WHERE`
conjuncts: each conjunct's text (lowercased, whitespace collapsed) and its
`cursor_columns`, the columns it compares with a bind parameter, or an interpolation
recovered from a template literal, as a keyset cursor
(`id > $1`, `(a, b) > ($1, $2)`, or `($1 IS NULL OR id > $1)`; parentheses are
transparent), with the `cursor_bound` (`Lower` for `id > $1`, `Upper` for `id < $1`) that
tells a window (`id >= $1 AND id < $2`) from a one-sided walk. A CTE body sees only the CTEs declared before it (all of them under
`WITH RECURSIVE`), so a body's table named like a later CTE is still a table. The `literal-limit` and
`keyset-only-sweep` shapes of `postgres-sql-shape-policy` consume them; the `LIMIT`
definition is shared with the row-bound facts.

`SchemaCatalog::unique_keys(table)` returns those key column sets: valid, ready, live,
immediate, non-partial unique or primary indexes whose keys are all plain columns, and
`SchemaCatalog::column_is_not_null(table, column)` says whether a column cannot hold NULL
(`ctid` never does), which decides whether an `IS NOT DISTINCT FROM` pin identifies a row.
`postgres-bounded-statements` consumes all three.

## Query annotation facts

`sql_requires_query_annotation(sql)` reports whether executed SQL is missing a
leading `/* name */` block comment. `BEGIN` / `COMMIT` / `ROLLBACK` are
exempt, including when they already carry a leading block comment. Line
comments (`-- name`) and empty `/* */` comments are not annotations.
`postgres-require-query-annotation` consumes this helper.

## Schema catalog model

`SchemaCatalog::load` reads a repository-relative schema catalog. The catalog
format is the one `no-mistakes postgres catalog` writes
([`postgres catalog`](cli/postgres.md)): no-mistakes owns both the format and its
generator, and this section is the only catalog contract. There is no external
snapshot producer, and the loader never accepts another tool's snapshot shape.
The file must state `formatVersion: 2` and its `coverage`, which is `complete`
(every fact below) or `ordering` (only what conflict and lock ordering need).
Nothing is assumed complete: a missing `coverage` is a load error. Every column
must carry `dataType`, so a missing type fails the load instead of becoming `""`.
An invalid catalog names the failing field, as in
`schemaCatalogPath <path> has an invalid schema: tables.accounts.columns.id:
missing field `dataType``, and says to generate the catalog with
`no-mistakes postgres catalog`. Catalogs accept strict JSON syntax with `//` and
`/* */` comments (JSONC); trailing commas and other JSONC extensions remain invalid.
Apart from those required fields, a missing field takes its default so a
hand-written fixture can state only the facts it tests; the generator always writes
all of them.

`postgres-finite-text-columns` reads column types, foreign keys, and `CHECK`
definitions from the catalog. `postgres-array-columns` reads column `data_type`
values and enum names from it. A rule that needs more than ordering facts rejects an
ordering catalog with `schemaCatalogPath <path> has ordering-only coverage; this
rule requires a complete schema catalog`; only `postgres-conflict-ordering` and
`postgres-lock-ordering` accept it. Both keep resolving arbiters from valid, ready,
unique or primary btree indexes only. `tables()`, `table(name)`, `functions()`,
`enums()`, and `views()` expose the rest of the catalog. Iterators are in
name order. Function order follows the catalog key. Tables are indexed by
normalized name, so `table(name)` and `relation(name)` find a quoted key such as
`"Order Items"` by its quoted or unquoted spelling.

`SCHEMA_CATALOG_RULE_IDS` selects which configured rules load a catalog.
It is a separate list from `PREPARED_EMBEDDED_SQL_RULE_IDS`, so a schema-only
rule can append its id without declaring an embedded-SQL executor.

The generator selects relations with one policy for both coverages: partition
children, extension-owned objects, internal triggers, PostgreSQL 18 NOT NULL
constraints, aggregates and window functions, sequences, TOAST tables, temporary
tables and foreign tables (unsupported) are not catalog facts. Types render relative
to the selected schema, so an enum column's `dataType` equals its enum name.
Identifier conventions: table, index, constraint, trigger, enum and function
keys are SQL identifiers, quoted only where PostgreSQL requires it. Keys of `columns`,
the column lists of `primaryKey` and `foreignKeys`, and index key `column` are raw
names. Unique-constraint `columns` and index key `expression` are written as SQL.
A function key is its name plus identity arguments, such as `over(a integer)`.
`ordinalPosition` is the 1-based position among live columns. The text from
`pg_get_functiondef`, `pg_get_viewdef`, `pg_get_constraintdef`, `pg_get_triggerdef`
and `format_type` is PostgreSQL's own and can differ between major versions.

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

The shape-policy request opts into `CheckFactPlan.postgres_fragments`. Its
prepared projection parses each distinct unexecuted builder text once, retains
each builder origin, and reuses the same statement facts for repeated consumers.
Executed fragments use the prepared executor statements instead. Standalone
and aggregate checks use this same request-owned preparation. Other rules do
not opt into fragment parsing.

Ordered CREATE and ADD COLUMN events retain `if_not_exists` so catalog consumers can preserve existing definitions when a migration retries DDL.

The shared migration comparator orders directories lexically and filenames by
their first numeric run. Live write catalogs retain qualified table lifecycle
identities and resolve unqualified DML only when the surviving relation is unique.
Unqualified ALTER and DROP share the same temporary-table preference and unique
suffix resolution; ambiguous lifecycle names do not alter a definite relation.

`table_events_collected` distinguishes a collected empty executed-event stream
from legacy Rust facts supplied without events. Broad table/column policy facts
still inspect routine bodies. Executed events retain top-level and immediate DO
DDL, excluding dormant function/procedure definitions and conditional branches.

ADD COLUMN events retain `table_if_exists` alongside column-level `if_not_exists`.

`relation_key` encodes identifier components without conflating quoted dots with
qualification; `table` retains its legacy decoded spelling. CREATE events retain
`temporary`, and live write catalogs project those relations in `pg_temp`.

Executed table events discard changes rolled back within the SQL source, including
`ROLLBACK TO SAVEPOINT`. `COMMIT`, transaction `END`, and released savepoints
retain their changes. Broad policy facts continue to describe rolled-back DDL.
`COMMIT AND CHAIN` and `ROLLBACK AND CHAIN` leave the following transaction active
for the next event group. `ABORT AND CHAIN` is the rollback synonym, including
optional WORK or TRANSACTION modifiers. A plain PL/pgSQL `RETURN` makes later
statements in that block non-definite, including after a conditional branch; `RETURN NEXT` and
`RETURN QUERY` continue execution in set-returning routines.

A variable assigned in conditional/loop/exception scope becomes opaque to later
definite EXECUTE recovery. Its syntactic SQL remains available to broad policy
facts, including assignments in dormant PL/pgSQL routines, without producing
executed table events.

`format()` recovery keeps `%I` and `%s` statements in broad schema facts using
placeholder names, but does not project their synthetic relation identities into
definite live-table history, including when the recovered SQL is assigned or
copied through a variable. `%L`, escaped `%%`, and format calls without runtime
placeholders remain concrete for live schema events.

Generated-column write checks declare their migration schema, SQL query files, and
executor profiles before the shared request fact pass. The statement pass records
INSERT, UPDATE, and MERGE target columns and positional widths without consulting
a catalog. Rule consumers borrow those facts and resolve the protected columns
after the ordered migration projection; they do not parse SQL again. Query-file
`include` scope is independent of migration `sqlInclude`.

## Generated schema catalogs

[`postgres catalog`](cli/postgres.md) generates the tool-owned JSON directly from
PostgreSQL, without an application snapshot producer. Complete coverage is accepted
by every catalog check. Explicit ordering coverage is accepted only by conflict/lock
ordering checks; full-schema checks reject it. For one database, conflict and lock
ordering report identical findings from either coverage.

Prepared row-bound facts are explicit demand. `CheckFactPlan.postgres_bounds` is
requested by `postgres-bounded-statements` and retained when plans are merged.
Other statement consumers and unexecuted builder fragments skip this projection;
adding bound demand preserves all existing statement fields. SQL parsing and source
reads remain owned by the same request-scoped preparation pass. Direct public
statement extraction retains its complete bound facts.

Temporary tables and views created earlier in the same SQL source shadow unqualified
catalog relations until `DROP TABLE` or `DROP VIEW`. This includes `SELECT INTO TEMP`.
Temporary relations are unknown and bound no joined items; qualified permanent
relations retain their catalog identity. State resets for every SQL source.
