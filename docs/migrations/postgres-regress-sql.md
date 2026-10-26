# PostgreSQL regression SQL

Schema checks distinguish analyzed statements, recognized unsupported PostgreSQL
syntax, and malformed or unknown SQL. Complete recognized unsupported statements
produce advisories under the default policy. Explicit `options.unanalyzable`
(`report`, `advise`, or `ignore`) retains its configured meaning. A command prefix
or balanced parentheses alone do not prove a statement valid.

The PostgreSQL 17 regression witnesses include utility commands such as VACUUM,
REINDEX, CLUSTER, cursor movement, prepared transactions, transaction settings,
COPY, EXPLAIN, privileges, and constraint comments. Recognition validates the
complete command, including required names, clauses, and argument lists. These
utility statements do not invent schema facts. Malformed near misses remain
findings under the default policy.

An invalid UTF-8 SQL file produces a file-local diagnostic at line 1. Save the file
as UTF-8 to analyze its contents. Other selected files still contribute schema
facts and policy findings. Encoding diagnostics obey the same explicit
unanalyzable policies as other SQL diagnostics; the source store retains the
failed read for the invocation.

A CREATE RULE action list can contain semicolons. The list remains part of that
rule's statement, so a later action is not reported as an orphan statement.
Unsupported rule grammar still receives its own diagnostic. Ordinary malformed
parentheses do not prevent recovery of subsequent migration statements.

Saved witnesses come from PostgreSQL `REL_17_STABLE` revision
`bd807012669ed8573317562b78c1221e4fabe373`, `src/test/regress/sql`.
The acceptance boundary is syntax, so a statement can be recognized even when
PostgreSQL later rejects its table, role, option value, or other semantic state.
The fixtures also retain malformed grammar controls.

Full regression statements with index opclass parameters, quantified `VALUES`
in a view, XML table columns, `TABLESAMPLE` with a null repeatable seed,
recursive CTE `SEARCH`, and a whole-row cast in a CTE-backed `MERGE` receive
the same syntax-based recognition. Recognition validates the original clauses
before checking a parser copy; malformed variants remain findings.
