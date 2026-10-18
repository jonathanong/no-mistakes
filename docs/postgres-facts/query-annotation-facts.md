<!-- cspell:ignore functiondef viewdef constraintdef triggerdef -->

## Query annotation facts

`sql_requires_query_annotation(sql)` reports whether executed SQL is missing a
leading `/* name */` block comment. `BEGIN` / `COMMIT` / `ROLLBACK` are
exempt, including when they already carry a leading block comment. Line
comments (`-- name`) and empty `/* */` comments are not annotations.
`postgres-require-query-annotation` consumes this helper.
When a call has complete variants, every version must have an annotation (or
be an exempt transaction statement). Calls without variants retain the rule's
separate prepared helper-prefix traversal, which can prove a leading
annotation even when an opaque tail prevents full SQL recovery.

[PostgreSQL fact sources](../postgres-facts.md)
