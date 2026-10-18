<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

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

[PostgreSQL fact sources](../postgres-facts.md)
