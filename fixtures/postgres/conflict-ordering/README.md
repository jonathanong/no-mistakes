# `postgres-conflict-ordering` fixtures

These cases use the versioned PostgreSQL schema-snapshot shape. They prove that a
multi-row UPSERT follows the resolved unique-index key order, including an expression index, and
that targetless or multiply inferred arbiters fail closed. Opaque executor
arguments fail closed as `unanalyzable-sql` unless `unanalyzableSql: ignore`.
