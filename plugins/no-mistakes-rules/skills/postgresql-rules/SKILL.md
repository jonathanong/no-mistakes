---
name: postgresql-rules
description: "MUST NOT / SHOULD NOT rules for PostgreSQL: queries, locks, transactions, schema, migrations, and partitioning. Use when writing or reviewing SQL, a migration, a repository/data-access function, or a query plan."
---

# PostgreSQL rules

MUST NOT is a defect. SHOULD NOT needs a stated, measured reason.

## Queries

- MUST NOT build SQL text or identifiers from input; bind values and pick identifiers from a closed allowlist.
- MUST NOT run an unbounded `SELECT`, `UPDATE`, or `DELETE` in a job or endpoint; add a `LIMIT` or required predicates.
- MUST NOT paginate with `OFFSET`; use keyset `(key, id) < ($1, $2)` ordered the same way.
- MUST NOT trust a client page size; clamp it to a static ceiling.
- MUST NOT query a protected relation without its required predicates (tenant, soft-delete, partition key).
- MUST NOT issue one query per item in a loop, `.map`, or `Promise.all`; write the batch query and make the single-item call wrap it.
- MUST NOT use `NOT IN (SELECT …)`; NULLs silently drop rows. Use `NOT EXISTS`.
- MUST NOT put volatile functions (`now()`, `random()`) in predicates or change detection; compute the bound once and bind it.
- MUST NOT write non-idempotent `INSERT`s on retried paths; use `ON CONFLICT`.
- MUST NOT write generated or trigger-maintained columns.
- SHOULD NOT use `SELECT *`, `alias.*`, or `RETURNING *`; list columns.
- SHOULD NOT use `COUNT(*) > 0` or `GROUP BY` to test existence or dedupe; use `EXISTS`.
- SHOULD NOT cast or wrap an indexed column in a predicate (UUIDs, timestamps); compare the raw column to a bound.
- SHOULD NOT add `MATERIALIZED` CTEs, `LATERAL`, `UNION`, `DISTINCT`, or candidate pools unless `EXPLAIN ANALYZE` at realistic size beats the plain query.
- SHOULD NOT send SQL without an operation-name comment.

## Locks, transactions, connections

- MUST NOT lock rows you do not write (`FOR UPDATE` on read sets, parents, globals).
- MUST NOT lock or upsert several rows in unspecified order; `ORDER BY` the key, or use `SKIP LOCKED` for competing workers.
- MUST NOT scatter manual `BEGIN`/`COMMIT`/`ROLLBACK`; use one transaction helper that owns cleanup.
- MUST NOT hold a transaction or pooled client across an external call or unrelated work; release clients in `finally`.
- MUST NOT `Promise.all` statements on one transaction client expecting parallelism; merge them into one statement.
- MUST NOT use advisory locks.
- MUST NOT read from a replica for read-after-write, locking, or lag-sensitive reads.
- MUST NOT add `BEGIN`/`COMMIT` inside a migration whose runner already wraps it in a transaction.
- MUST NOT run `EXPLAIN ANALYZE` on a write outside a rolled-back transaction.
- MUST NOT fold away a statement without keeping the lock or snapshot guarantee it gave.

## Schema and migrations

- MUST NOT add a foreign key without an index on the referencing column.
- MUST NOT store relations as arrays, JSON ids, or type/id pairs; use foreign keys or child rows.
- MUST NOT leave a `NOT VALID` constraint without a later `VALIDATE CONSTRAINT`.
- MUST NOT exceed 63-byte identifiers; PostgreSQL truncates them silently.
- MUST NOT put `now()` or `timestamptz + interval` in generated or index expressions; they are not immutable.
- MUST NOT run DDL or session settings from runtime or seed SQL; keep them in migrations.
- MUST NOT hand-sync a status column beside lifecycle timestamps; derive one from the other.
- MUST NOT update or delete append-only history rows; reject it with a trigger.
- SHOULD NOT omit `ON DELETE`, constraint names, or `COMMENT ON` for tables and columns.
- SHOULD NOT use free text for a finite set; use an enum or lookup table.
- SHOULD NOT use natural text keys as primary or foreign keys.
- SHOULD NOT add an index that is a prefix of another, or copy near-identical trigger functions.
- SHOULD NOT write `updated_at` from application code; maintain it with a trigger.
- SHOULD NOT keep durable rows without a retention window (user content excepted).

## Partitioning and UUIDv7

- MUST NOT partition before retention and query predicates show it helps, or partition short-lived rows.
- MUST NOT use `HASH` partitioning for time-ordered data; use `RANGE` on the time-ordered id plus a default partition.
- MUST NOT read partition children directly; query the parent with partition-key bounds.
- MUST NOT omit the partition key from unique constraints or reader predicates; check pruning in the plan.
- MUST NOT extract timestamps from ids to prune; generate id bounds from time.
- MUST NOT assume UUIDv7 order is commit order; re-read an overlapping window instead of trusting a high-water mark.
- MUST NOT attach a populated range or drop a partition without checking the default partition, retention, and holds.
