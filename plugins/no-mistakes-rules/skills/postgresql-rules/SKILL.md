---
name: postgresql-rules
description: "Load before writing or reviewing anything that sends SQL to PostgreSQL: a query through pg/node-postgres, Kysely, Drizzle, Prisma, or psql; a repository or data-access function; a migration; an index; or a query plan. Lists the defects to avoid and the patterns that need a stated reason: queries, locks, transactions, schema, migrations, partitioning."
---

# PostgreSQL rules

MUST NOT is a defect. SHOULD NOT needs a stated, measured reason.

## Queries

- MUST NOT build SQL text or identifiers from input; bind values and pick identifiers from a closed allowlist.
- MUST NOT run an unbounded `SELECT`, `UPDATE`, or `DELETE` in a job or endpoint; add a `LIMIT` or required predicates.
- SHOULD NOT paginate a growing result with `OFFSET`; its cost grows with page depth and concurrent inserts shift rows between pages. Use keyset `(key, id) < ($1, $2)` ordered the same way.
- MUST NOT trust a client page size; clamp it to a static ceiling.
- MUST NOT derive a next-page flag from whether the page is full when rows are filtered or dropped in application code after the query; a short or empty page then hides later rows. Set it from whether the underlying read has more rows (for example, fetch `LIMIT n + 1` before filtering).
- SHOULD NOT let a paginated read scan unbounded work for one page (for example, an ever-wider time window until a row matches); bound the work per page and return the cursor, since unbounded scans time out before the client reaches results or the end.
- MUST NOT query a protected relation without its required predicates (tenant, soft-delete, partition key).
- SHOULD NOT issue one query per item for an input-sized batch when one query can do the same work.
- MUST NOT make statement shape depend on row count (one `UNION ALL` arm or one bind list per item); pass an array.
- SHOULD NOT look up the same row more than once in one statement through separate joins or scalar subqueries keyed by the same expression; each extra lookup adds another probe per output row. Join it once and project every field from that row.
- SHOULD NOT join or subquery a view that contains joins or subqueries only to read one or two base-table columns; the view's joins and subqueries still run for every output row. Join the base table with its required predicates.
- MUST NOT use `NOT IN (SELECT …)`; NULLs silently drop rows. Use `NOT EXISTS`.
- SHOULD NOT put volatile functions (`random()`, `clock_timestamp()`) in predicates unless per-row evaluation is intended; they are evaluated per row.
- MUST NOT write non-idempotent `INSERT`s on retried paths; use `ON CONFLICT` on a key the retry repeats, not a freshly generated one.
- MUST NOT assign a non-DEFAULT value to a generated column.
- SHOULD NOT write trigger-maintained columns unless the schema allows an explicit override.
- SHOULD NOT `UPDATE t SET c = c` to fire a trigger; the assignment still writes a row version.
- MUST NOT read `COUNT(expr)` as a count of true rows; it counts non-NULL values. Use `COUNT(*) FILTER (WHERE expr)`.
- SHOULD NOT use `COUNT(*) > 0` or `GROUP BY` to test existence or dedupe; use `EXISTS`.
- SHOULD NOT cast or wrap an indexed column in a predicate (UUIDs, timestamps); compare the raw column to a bound.
- SHOULD NOT keep predicate arms that cannot match (`AND FALSE` legs, `col IS NULL` on a `NOT NULL` column, comparing an id with foreign keys to other tables).
- SHOULD NOT add `MATERIALIZED` CTEs, `LATERAL`, `UNION`, `DISTINCT`, or candidate pools unless `EXPLAIN ANALYZE` at realistic size beats the plain query.
- SHOULD NOT upsert by overwriting every column; it resets an existing row to the inserter's values. Use `DO UPDATE SET` on the columns you own, or `DO NOTHING` then read the row.
- SHOULD NOT upsert with `DO UPDATE SET k = EXCLUDED.k` on the conflict key only to get `RETURNING`; it writes a row version on every call. Use `DO NOTHING`, then read the row.

## Locks, transactions, connections

- SHOULD NOT take row locks unrelated to the work or its serialization invariant.
- MUST NOT lock or upsert several rows in unspecified order; `ORDER BY` the key, or use `SKIP LOCKED` for competing workers.
- MUST NOT hold a transaction or pooled client across an external call or unrelated work; release clients in `finally`.
- MUST NOT `Promise.all` statements on one transaction client expecting parallelism; merge them into one statement.
- MUST NOT take session-scoped advisory locks on a connection that returns to a shared pool; they outlive the transaction and stay with the connection.
- MUST NOT read from a replica for read-after-write, locking, or lag-sensitive reads.
- MUST NOT add `BEGIN`/`COMMIT` inside a migration whose runner already wraps it in a transaction.
- MUST NOT run `EXPLAIN ANALYZE` on a write outside a rolled-back transaction.
- MUST NOT fold away a statement without keeping the lock or snapshot guarantee it gave.
- MUST NOT write an ORM lock that never executes; lazy querysets take no lock until evaluated, and some ORMs drop the lock on bulk delete.
- MUST NOT lock related tables in different orders on different paths; lock parent before child everywhere.
- SHOULD NOT insert a foreign-key child row in the same transaction before updating a column of the same parent covered by a unique index usable as a foreign-key target. Concurrent transactions can each take `FOR KEY SHARE` during the foreign-key check, then deadlock when each update needs `FOR UPDATE` strength. Update the parent first, or acquire the stronger parent lock before the insert.
- MUST NOT pick which parent to lock from an unlocked read of the child's foreign key; lock that parent, then lock the child and re-check the key, retrying if it moved.
- SHOULD NOT ship a job that locks contended rows without a recovery path for deadlocks, such as bounded retries with backoff.
- MUST NOT keep using a transaction after a statement fails unless it ran in a savepoint you rolled back to; every later statement fails with `25P02`.
- MUST NOT run logging or metrics code that queries the database on the transaction's connection; its failure aborts the transaction.
- MUST NOT rely on `SERIALIZABLE` to stop a racing writer unless each transaction reads rows the other writes. A guard (hold, ban, cancel) must read the rows the writer would change; otherwise SSI orders the writer first with no `40001`.
- SHOULD NOT add `FOR UPDATE` to a subquery or pre-select whose rows the same `UPDATE` or `DELETE` already locks and whose values you do not read; it only adds a second lock order. A `SKIP LOCKED` claim subquery is fine.

## Schema and migrations

- SHOULD NOT add a foreign key without an index on the referencing column; deleting or updating a parent row scans the child table under lock.
- SHOULD NOT store relations as arrays, JSON ids, or type/id pairs; use foreign keys or child rows.
- MUST NOT leave a `NOT VALID` constraint without a later `VALIDATE CONSTRAINT`.
- MUST NOT exceed 63-byte identifiers; PostgreSQL truncates them silently.
- MUST NOT put `now()` or `timestamptz + interval` in generated or index expressions; they are not immutable.
- SHOULD NOT change persistent application schema from runtime or seed SQL. Use migrations for schema changes and scope session settings to the work that needs them.
- SHOULD NOT raise `statement_timeout` or idle timeouts to make slow work fit when you can split, reorder, or shrink the work. An indivisible step, such as building an index on a large table, may use a higher timeout scoped to that step.
- MUST NOT update append-only history rows, or delete them outside a retention policy.
- SHOULD NOT add a `NOT NULL`, unique, or check constraint to an existing table without checking whether existing rows satisfy it.
- MUST NOT rename or renumber a migration after it has run in any shared environment.
- MUST NOT edit a migration after it has run in any shared environment.
- SHOULD NOT create the same function or index in more than one migration; only the last definition survives.
- MUST NOT run code that depends on a migration before that migration is applied; order the deploy so the migration runs first, or expand first.
- SHOULD NOT build an index on a populated table without `CONCURRENTLY`, outside the migration's transaction; a failed build leaves an `INVALID` index to drop before retrying. A partitioned parent cannot build concurrently: build each partition's index concurrently, then create the parent index `ON ONLY` and attach them.

## Partitioning

- SHOULD NOT partition a table without evidence that partitioning helps retention, query performance, or maintenance.
- SHOULD NOT read individual partitions in ordinary application queries. Query the parent with partition-key bounds when available.
- SHOULD NOT omit partition-key bounds from queries that can use partition pruning. Check pruning in the plan.
- SHOULD NOT infer commit order from UUIDv7 values alone.
