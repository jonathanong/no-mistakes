<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

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

[PostgreSQL fact sources](../postgres-facts.md)
