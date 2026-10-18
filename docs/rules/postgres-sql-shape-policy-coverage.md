# postgres-sql-shape-policy coverage

The opted-in `not-in-subquery` shape also checks `HAVING`, aggregate `FILTER`,
and mutation predicates, including `MERGE ON` and `WHEN … AND` conditions.
MERGE source queries and subqueries in its update/insert actions are also
checked. Nested `NOT` operators are evaluated together so an
even number of negations does not report an allowed `IN` predicate.

`count-for-existence` recognizes transparent casts of scalar `COUNT(*)` values
and the built-in `COUNT(*)` or `pg_catalog.count(*)` aggregate. It deliberately
skips `COUNT(expression)`: that count excludes NULL expressions, so replacing
it with a plain `EXISTS` would change the result when matching rows contain only
NULL values. Schema-qualified custom functions and window counts are excluded.
It also skips aggregate subqueries with `HAVING`, `LIMIT`/`OFFSET`, or `FETCH`,
which can return no row; a scalar subquery then yields NULL instead of a zero
count.
Findings recommend `EXISTS` for presence checks and `NOT EXISTS` for zero-row
checks, preserving the predicate.
Surrounding `NOT` operators are folded into the count comparison polarity;
use the recommended EXISTS form for the complete negated comparison.
Ungrouped bare counts in `HAVING` are existence probes and are reported.
Grouped bare counts in `HAVING` remain allowed. For example,
`GROUP BY account_id HAVING COUNT(*) > 0` tests each group rather than a scalar
existence probe.

Locations use parsed expression spans, including SQL comments, Unicode names,
`!=` operator spelling, repeated statements, and embedded SQL declaration
lines. Line-specific suppressions therefore apply at the reported expression.
Unanalyzable SQL findings use a target from the enabled `bannedShapes`.

A single effective empty grouping set (`GROUP BY ()` or
`GROUPING SETS (())`) retains global aggregate semantics, so its count existence
probes are checked. `GROUPING SETS` keeps duplicate sets by default, and
`ROLLUP`/`CUBE` can expand empty inputs into repeated empty sets; those queries
can return multiple rows and are not treated as scalar count queries. The
`DISTINCT` grouping modifier collapses duplicate sets. Aggregate FILTER
predicates contribute one fact per occurrence.

Embedded findings use the literal's physical source line, including cooked newline
escapes, line continuations, and literals that begin below their declaration.
The Rust facts retain original SQL columns in `SqlSelectFact.not_in_columns` and
`SqlCountExistenceFact.column`; embedded calls expose compact source-line
transitions through `EmbeddedSqlCall.sql_source_positions`.
PostgreSQL integer limits support digit separators, including one immediately after an octal or binary radix prefix (for example, `LIMIT 0o_1_755` or `LIMIT 0b_1_0`). Their decoded row counts follow the same `literal-limit` policy as decimal limits.

Radix-prefix separators also survive lenient migration recovery when unrelated
statements require the fallback parser; valid statements retain their source
locations and numeric row caps.
Numeric hexadecimal limits such as `LIMIT 0xF_F` decode to the row count 255.
Hexadecimal string syntax such as `LIMIT X'FF'` remains a non-literal expression;
the classifier uses the original source spelling to distinguish them, including
when Unicode characters precede the limit on its source line.
Valid radix integers above the supported unsigned row-cap range remain parseable
as numeric expressions. Their cap is classified as unknown, so they do not
prevent collection of other statement facts.
Adjacent radix tokens are repaired before SQL parsing, including projections
that would otherwise be accepted as implicit aliases. Whitespace-separated
aliases and quoted names retain their existing meaning.

Numeric hexadecimal tokens retain their lexical identity through lenient encoded
migration recovery, including concatenated strings and `chr()` reconstruction.
Quoted hexadecimal strings remain non-literal caps after the same recovery.

Uppercase hexadecimal prefixes such as `0XFF` have the same numeric meaning in
strict and recovered SQL. Whitespace-separated aliases remain aliases.

Negating a recognized keyset comparison reverses its bound (`NOT (id <= $1)`
is a lower-bound cursor). It still walks a key range; combining it with an
opposite upper bound forms a selective window. Arbitrary negated predicates and
optional-cursor disjunctions with an odd number of negations remain outside this
comparison proof. An even number of `NOT` operators preserves the original
optional cursor, including its NULL-switchable whole-table case.

For `keyset-only-sweep`, a zero-row page remains empty through parentheses, unary
signs and standard PostgreSQL numeric casts (`LIMIT +0`, `LIMIT 0::bigint`).
Unquoted catalog-qualified numeric casts such as `LIMIT 0::pg_catalog.int8` also
prove an empty page. Application-schema types, quoted custom type spellings, and
casts through nonnumeric intermediate types stay conservative. This semantic check does not change
`literal-limit` classification: cast or unary expressions remain non-literal.

[Back to the rule](postgres-sql-shape-policy.md)
