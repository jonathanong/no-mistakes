<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

## Locking-select facts

`extract_locking_select_metadata(sql)` parses PostgreSQL SQL and returns one
`LockingSelectMetadata` per `SELECT` that uses `FOR UPDATE`:

- `has_multi_row_predicate` — the locked select's `WHERE` uses `IN` or `= ANY`
- `has_order_by` — the locked query has `ORDER BY`
- `skips_locked_rows` — the lock uses `SKIP LOCKED`
- `tables` — the schema-preserving base relations selected by the lock clause;
  `None` when a relation cannot be resolved (an `OF` target that is not one base
  table, or a derived relation locked without `OF`)
- `table_qualifiers` — the schema, base-name, and alias qualifiers valid for
  each locked relation
- `pinned_columns` — per base relation in the `FROM`, the columns a top-level `AND`
  equality (in `WHERE` or an inner join's `ON`) pins to a literal, placeholder, or
  recovered interpolation (`extract_locking_select_metadata_with_placeholders`
  takes the interpolation positions)
- `join_equalities` — equalities between columns of two distinct base relations;
  the lock-ordering rule pairs both with a catalog unique key to prove a
  single-row lock
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

[PostgreSQL fact sources](../postgres-facts.md)
