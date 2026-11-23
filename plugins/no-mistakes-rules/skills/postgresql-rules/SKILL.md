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
- MUST NOT derive a next-page flag from whether the page is full when rows can be filtered or capped after the `LIMIT`; a short or empty page then hides later rows. Set it from whether the read can continue.
- SHOULD bound a run of empty pages a paginated read can return (for example by a time window), so a client following the cursor reaches results or the end.
- MUST NOT query a protected relation without its required predicates (tenant, soft-delete, partition key).
- SHOULD NOT issue one query per item for an input-sized batch when one query can do the same work.
- MUST NOT use `NOT IN (SELECT …)`; NULLs silently drop rows. Use `NOT EXISTS`.
- SHOULD NOT put volatile functions (`random()`, `clock_timestamp()`) in predicates unless per-row evaluation is intended; they are evaluated per row.
- MUST NOT write non-idempotent `INSERT`s on retried paths; use `ON CONFLICT` on a key the retry repeats, not a freshly generated one.
- MUST NOT assign a non-DEFAULT value to a generated column.
- SHOULD NOT write trigger-maintained columns unless the schema allows an explicit override.
- MUST NOT read `COUNT(expr)` as a count of true rows; it counts non-NULL values. Use `COUNT(*) FILTER (WHERE expr)`.
- SHOULD NOT use `COUNT(*) > 0` or `GROUP BY` to test existence or dedupe; use `EXISTS`.
- SHOULD NOT cast or wrap an indexed column in a predicate (UUIDs, timestamps); compare the raw column to a bound.
- SHOULD NOT add `MATERIALIZED` CTEs, `LATERAL`, `UNION`, `DISTINCT`, or candidate pools unless `EXPLAIN ANALYZE` at realistic size beats the plain query.
- SHOULD NOT upsert by overwriting every column; it resets an existing row to the inserter's values. Use `DO UPDATE SET` on the columns you own, or `DO NOTHING` then read the row.

## Locks, transactions, connections

- SHOULD NOT take row locks unrelated to the work or its serialization invariant.
- MUST NOT lock or upsert several rows in unspecified order; `ORDER BY` the key, or use `SKIP LOCKED` for competing workers.
- MUST NOT hold a transaction or pooled client across an external call or unrelated work; release clients in `finally`.
- MUST NOT `Promise.all` statements on one transaction client expecting parallelism; merge them into one statement.
- MUST NOT take session-scoped advisory locks on a pooled connection; they outlive the transaction and stay with the connection.
- MUST NOT read from a replica for read-after-write, locking, or lag-sensitive reads.
- MUST NOT add `BEGIN`/`COMMIT` inside a migration whose runner already wraps it in a transaction.
- MUST NOT run `EXPLAIN ANALYZE` on a write outside a rolled-back transaction.
- MUST NOT fold away a statement without keeping the lock or snapshot guarantee it gave.
- MUST NOT write an ORM lock that never executes; lazy querysets take no lock until evaluated, and some ORMs drop the lock on bulk delete.
- MUST NOT lock related tables in different orders on different paths; lock parent before child everywhere.
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
- MUST NOT run code that depends on a migration before that migration is applied; order the deploy so the migration runs first, or expand first.
- SHOULD NOT build an index on a populated table without `CONCURRENTLY`, outside the migration's transaction; a failed build leaves an `INVALID` index to drop before retrying. A partitioned parent cannot build concurrently: build each partition's index concurrently, then create the parent index `ON ONLY` and attach them.

## Partitioning

- SHOULD NOT partition a table without evidence that partitioning helps retention, query performance, or maintenance.
- SHOULD NOT read individual partitions in ordinary application queries. Query the parent with partition-key bounds when available.
- SHOULD NOT omit partition-key bounds from queries that can use partition pruning. Check pruning in the plan.
- MUST NOT infer commit order from UUIDv7 values alone.
