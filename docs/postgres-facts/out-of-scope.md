<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

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

[PostgreSQL fact sources](../postgres-facts.md)
