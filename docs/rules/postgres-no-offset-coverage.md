# postgres-no-offset statement coverage

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

## Statement coverage and locations

The shared SQL pass records every OFFSET in source order, including INSERT,
UPDATE, DELETE, COPY queries, CREATE TABLE AS, views, CTEs, RETURNING, ON CONFLICT,
and subqueries inside CASE, arrays, functions, predicates, and ordering. Routine
declarations and non-analyzing EXPLAIN plans are skipped; EXPLAIN ANALYZE queries
are executed and checked, including the parenthesized ANALYZE option.
COPY FROM STDIN payload rows are data; queries after
its `\.` terminator are still checked. Nested comments, escaped E strings,
dollar quotes, and Unicode identifiers retain their SQL meaning.

Each finding points to the OFFSET keyword. Shared SQL constants point to their
declaration lines, and embedded multiline queries retain line-specific
suppression. Multiple offsets on the same line remain separate findings.
The first occurrence in a file keeps target `offset`; later occurrences use
`offset#2`, `offset#3`, and so on. The Rust statement-fact contract exposes these
occurrences as `SqlStatementFileFacts.offset_uses`, with line, column, and
`OffsetUse` kind. `sql_offset_uses` and `sql_file_offset_uses` use this same AST
visitor and preserve source order. Bare `*.sql` patterns match SQL basenames;
`sqlInclude: []` continues to opt out of SQL file scanning.

Recovered literals and template quasis retain a compact physical-line map in
`EmbeddedSqlCall.sql_source_positions` (`EmbeddedSqlSourcePosition`). Cooked
newline escapes, line continuations, multiline interpolations, and initializers
that begin below their declaration preserve the actual OFFSET source line.
Static `.append()` composition retains each appended literal or bound fragment's
physical position, including placeholder renumbering. A straight-line same-file helper that receives and returns a typed
`SQLStatement` parameter contributes its static appends to this rule, so an
appended `OFFSET` is checked normally. Unknown builder arguments and dynamic
appends retain the strict `unanalyzableSql` behavior. A local alias inside a typed helper can currently make its result unanalyzable;
only verified helper summaries retain complete SQL.
Reassignment, branching, and unknown escapes remain unanalyzable. Put line
suppression on the physical clause being suppressed. Existing directives on an executor call
also suppress that call's recovered clauses through the common suppression pass.
A directive on one executor call does not suppress another call that reuses
the same SQL constant or fragment. Repeated unsuppressed executions report the
shared OFFSET once; a directive on the shared OFFSET's physical line suppresses
that clause wherever it is used.
Prepared source failures retain their I/O kind; dispatch uses that captured
outcome without checking filesystem state again.
Source positions mark changes to the source-versus-SQL line offset. Between
positions, map a SQL line with `source_line + sql_line - position.sql_line`;
an empty position list retains the ordinary call/declaration line mapping.

[Back to the rule](postgres-no-offset.md)
