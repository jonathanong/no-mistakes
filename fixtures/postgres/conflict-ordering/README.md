# `postgres-conflict-ordering` fixtures

These cases use the versioned PostgreSQL schema-snapshot shape. They prove that a
multi-row UPSERT follows the resolved unique-index key order, including an expression index, and
that targetless or multiply inferred arbiters fail closed. Opaque executor
arguments fail closed as `unanalyzable-sql` unless `unanalyzableSql: ignore`.
`sql-template-strings` statement-level `.append()` mutation chains recover when
fragments are static; recovered non-INSERT queries stay clean, and recovered
INSERT/ON CONFLICT still gets the catalog ordering check. Conditional static
appends keep recovered SQL as dynamic so INSERT cannot pass a branch-only
order as if it always ran. Opaque append fragments likewise retain a recovered
leading statement as dynamic so SELECT/UPDATE stay outside this rule while
INSERT remains fail-closed.
