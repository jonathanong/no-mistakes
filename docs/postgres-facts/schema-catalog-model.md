<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

## Schema catalog model

`SchemaCatalog::load` reads a repository-relative schema catalog. The catalog
format is the one `no-mistakes postgres catalog` writes
([`postgres catalog`](../cli/postgres.md)): no-mistakes owns both the format and its
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
Generated catalogs retain nested partitioned parents and set `partitionOf` to
the immediate parent name, schema-qualifying it when the parent is outside the
catalog's schema. Ordinary partition leaves with only inherited key clones are
omitted; leaves with their own primary or foreign key are retained with only
their local constraints. Handwritten snapshots may also set `partitionOf` to
identify partition children; explicit key facts on a leaf are interpreted as
that leaf's own constraints.

`postgres-key-column-types` reads primary-key columns, foreign-key referencing
columns, enum types, and partition relationships. `postgres-finite-text-columns` reads column types, foreign keys, and `CHECK`
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

[PostgreSQL fact sources](../postgres-facts.md)
