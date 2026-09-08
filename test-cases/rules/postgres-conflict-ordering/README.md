# `postgres-conflict-ordering` fixtures

These cases use the committed Vouchington PostgreSQL schema-snapshot shape. They prove that a
multi-row UPSERT follows the resolved unique-index key order, including an expression index, and
that targetless or multiply inferred arbiters fail closed.
