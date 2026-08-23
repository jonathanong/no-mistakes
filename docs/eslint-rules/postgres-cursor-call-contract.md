# `no-mistakes/postgres-cursor-call-contract`

Require configured PostgreSQL cursor helpers to be called directly with SQL
that starts with a static `/* name */` annotation.

Why: cursor executors hold a client for the life of the stream. Aliases,
dependency containers, and dynamic SQL hide the statement from static review,
so idle-in-transaction and unlabeled query plans cannot be attributed to one
callsite.

Counterexample: `runCursor('SELECT 1')`, `const cursor = runCursor; cursor(sql)`,
or `export { runCursor } from '@db/cursors'`.

Fix: import the helper by name or namespace member and pass a string,
template, or a configured SQL tagged-template module (default
`sql-template-strings`) whose first static text is `/* rows */ …`. One
immutable `const` binding is allowed. Discarded `.append(...)` chains on that
binding are ignored.

Not in `configs.recommended` or `configs.strict`. Enable it with `modules` and
`executors` for the project. Optional `include` / `exclude` / `includeFiles`
scope the files; `annotation` replaces the default leading-comment regex;
`sqlTagModules` replaces the default SQL tag import.
