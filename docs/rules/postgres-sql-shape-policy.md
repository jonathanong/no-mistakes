# `postgres-sql-shape-policy`

Ban configured PostgreSQL SQL shapes that the parser can prove. The default
shape is `correlated-exists-set-operation`: a correlated
`EXISTS (SELECT … UNION [ALL] / INTERSECT / EXCEPT …)`. Postgres cannot turn
that set-operation body into a nested loop join, so it rebuilds the whole set
once per outer row. An uncorrelated `EXISTS (… UNION …)` is planned once as an
InitPlan and is not this shape.

The rule consumes dual-source statement facts (`CheckFactPlan.postgres_dml`),
including recoverable SQL returned from builders and fragments appended to a
known SQL builder (a recovered SQL binding or a parameter typed
`SQLStatement`). Arbitrary `.append(...)` receivers are not SQL builders.
Builder recovery honors the same trusted-tag and lexical-shadow rules as
executed SQL. Unresolved raw appended identifiers become a synthetic qualified
outer reference, so a builder that may append an outer column is conservatively
treated as possibly correlated. Identical executed and builder SQL is reported
once. Unparseable or dynamic SQL, including builder fragments, fails closed
unless `unanalyzableSql` is `ignore`.

```yaml
rules:
  - rule: postgres-sql-shape-policy
    scope: repository
    options:
      sqlInclude: ["**/*.sql"]
      bannedShapes:
        - correlated-exists-set-operation
        - not-in-subquery
        - count-for-existence
        - literal-limit
        - keyset-only-sweep
      shapeOptions:
        literalLimit:
          allowedValues: [1]
        keysetOnlySweep:
          nonSelectivePredicates: ['deleted_at IS NULL']
          ignoreTables: []
      unanalyzableSql: fail
```

`sqlInclude` defaults to `**/*.sql`. `bannedShapes` defaults to
`[correlated-exists-set-operation]`. `not-in-subquery`, `count-for-existence`,
Numeric digit separators in literal limits are decoded (`LIMIT 1_000` is `LIMIT 1000`).

`literal-limit` and `keyset-only-sweep` are opt-in. Unknown `bannedShapes` values
are a configuration error. `unanalyzableSql` defaults to `fail` (`fail` or `ignore`;
other values are a configuration error).
`importSpecifier` defaults to `@data-stores/psql`; `executorNames` defaults to
`[query, read, write]`.

Counterexample: correlated `EXISTS` wrapping `UNION ALL`.

```sql
SELECT 1 FROM posts
WHERE EXISTS (
  SELECT 1 FROM topics WHERE topics.post_id = posts.id
  UNION ALL
  SELECT 1 FROM tags WHERE tags.post_id = posts.id
);
```

Fix: keep the set operation uncorrelated, then test membership — for example
`post_id IN (SELECT candidate.post_id FROM ( … UNION ALL … ) candidate)` — or
use a single-arm `EXISTS` without a set operation.

```sql
SELECT 1 FROM posts
WHERE posts.id IN (
  SELECT candidate.post_id FROM (
    SELECT topics.post_id FROM topics
    UNION ALL
    SELECT tags.post_id FROM tags
  ) candidate
);
```

An uncorrelated `EXISTS` around a set operation, including a FROM-less
`SELECT EXISTS (… UNION …) AS alias` scalar probe, is allowed. An inner
placeholder on every arm does not make a correlated wrapping `EXISTS` safe.

Use `no-mistakes-disable-next-line postgres-sql-shape-policy` immediately
before the `EXISTS`, or `no-mistakes-disable-line` on that line, or
`no-mistakes-disable-file` when a whole file is an intentional exception.

## Why and when

Use this rule when correlated `EXISTS (… UNION …)` has caused per-outer-row
set-operation scans, and you want a fail-closed shape ban instead of a
table-specific allowlist.

## What it catches/requires

When `correlated-exists-set-operation` is banned, an `EXISTS` whose subquery
is a set operation is a finding when a qualified `table.column` in that
subquery names a relation that is not local to the subquery's FROM/WITH
(including SELECT-list and HAVING `EXISTS`). Set operations nested inside a
derived-table `FROM` of the `EXISTS` subquery are not this shape.

When `not-in-subquery` is banned, `NOT IN (SELECT …)` and
`NOT (… IN (SELECT …))` are findings. `NOT IN` of a value list and `IN (SELECT …)`
are not. When `count-for-existence` is banned, a built-in `COUNT(*)` compared
with 0 or 1 to test existence is a finding: `> 0`, `>= 1`, `<> 0`, `!= 0`,
`= 0`, `< 1`, `<= 0`, and the mirrored forms. The count may be a scalar
subquery with no `GROUP BY`, or a bare `COUNT(*)` in a SELECT list, WHERE, or
HAVING of a query with no `GROUP BY`.
Grouped `HAVING COUNT(*) > 0` and comparisons with any other number are not
findings. Lowercase quoted built-ins (`"count"` or `pg_catalog."count"`) are
recognized; quoted uppercase or custom-schema functions are excluded.

Correlation is a syntax heuristic: only qualified references count, an
aliased inner relation hides its base name, `schema.table.col` uses the table
component, UNION branches share one local-name set, and nested subquery
scopes are not tracked.

## Options and defaults

`include` / `exclude` select source files (empty include means all files).
`sqlInclude` defaults to `**/*.sql`. `bannedShapes` defaults to
`[correlated-exists-set-operation]`. Also accepted, and off unless listed:
`not-in-subquery`, `count-for-existence`, `literal-limit`, `keyset-only-sweep`.
Unknown `bannedShapes` values are a configuration error. `unanalyzableSql` defaults to `fail` (`fail` or `ignore`;
other values are a configuration error). `importSpecifier` defaults to
`@data-stores/psql`. `executorNames` defaults to `[query, read, write]`.

## Bounded iteration shapes

Two opt-in shapes flag background-job SQL that signals unbounded work.

`literal-limit` reports a `LIMIT` or `FETCH FIRST` whose count is an integer
literal that is not in `shapeOptions.literalLimit.allowedValues`. A batch size
written in SQL text cannot be tuned without a deploy, and it hides that the
statement is a batch. Bind it (`LIMIT $1`) so the batch size is explicit
configuration. Every query is checked, including CTEs and subqueries of
`UPDATE` and `DELETE`. A placeholder, an expression and `LIMIT ALL` are not
literals. Complete unexecuted SQL builders, including `SELECT 1` and `VALUES (1)`
without a `FROM` clause, retain their limit facts and source locations.
`FETCH FIRST ROW ONLY` counts as the literal `1`, reported at its
`FETCH` keyword (a count that is written is reported at the count). `FETCH … WITH
TIES` still writes a literal count, so it is reported too. Implicit `FETCH` clauses
in an `OFFSET` subquery and its outer query retain separate source locations;
a line suppression applies only to the clause on that line.

`keyset-only-sweep` excludes `DISTINCT`, `DISTINCT ON`, and grouped queries because
they page distinct groups rather than table rows. `SELECT ALL` preserves table rows
and remains eligible. With both shapes enabled, `literal-limit` still checks the
written counts of these queries; for example, `SELECT DISTINCT id FROM orders
ORDER BY id LIMIT 500` is reported for its literal limit, without a sweep finding.

Expanded lexicographic cursors such as `a > $1 OR (a = $1 AND b > $2)` count like tuple cursors when each equality prefix uses the same bind, each comparison has the same direction, and the keys follow the leading `ORDER BY` columns with a uniform sort direction. The OR arms may appear in any order; each must still form exactly one step in the same equality-prefix chain. Either side of each comparison may carry the bind, with the range direction inverted when the bind appears first. Built-in int4 cast spellings preserve bind identity across equality-prefix arms; other casts match only when their complete expressions are identical. For `PREPARE` statements with declared non-int4 parameter types, casts remain distinct because conversion to int4 can change the bound value.

Bind-only guards without column references, calls, or subqueries do not narrow a walk.

`keyset-only-sweep` reports a walk over a whole table by its key: a query with a
`LIMIT` over one base table (no join, grouping or set operation) ordered only by
plain columns of that table, where every top-level `WHERE` conjunct is either a
keyset cursor or a configured non-selective predicate. A cursor compares an `ORDER
BY` column with a bind parameter using `>`, `>=`, `<` or `<=`: `id > $1`,
`(created_at, id) > ($1, $2)`, or the optional form `($1::uuid IS NULL OR id > $1)`.
With no `WHERE` at all the walk is whole-table too. Any other conjunct narrows the
walk (a due timestamp, a parent id, `lease_expires_at IS NULL`), so the query is
allowed: a background job should choose rows that need work, such as a work-item row,
a dirty marker, a due timestamp or a parent id. A comparison with a literal or a
function call such as `now()` is not a cursor. An interpolation in an executor's
template literal (`` sql`… WHERE id > ${after} … LIMIT ${size}` ``) is a bind, like
`$1`. A query is a page only when it caps its rows, so `FETCH … WITH TIES`,
`FETCH … PERCENT` and `LIMIT 0` queries are not sweeps. Parentheses around a conjunct, an
operand or the whole `SELECT` (`(SELECT …) ORDER BY id LIMIT $1`) do not change what it is.
When the outer query has no `ORDER BY`, the one inside the parentheses orders the rows its
`LIMIT` counts (`(SELECT … ORDER BY id) LIMIT $1`).
A cursor bounds one side of the walk, so a lower and an upper bound on the ordered columns
together (`id >= $1 AND id < $2`) are a window, which narrows it; an optional bound
(`($1 IS NULL OR id >= $1)`) does not count, because a NULL bind switches it off. A constant `TRUE` or
`1 = 1` conjunct, which query builders seed a `WHERE` with, selects nothing, and a relation
with a `TABLESAMPLE` is restricted before it is ordered, so it is not a whole-table walk.
A bare `ORDER BY` name
that is also an output name means that output expression: `SELECT random() AS id … ORDER BY
id`, and `SELECT random() … ORDER BY random`, are not ordered by a column of the table. A
builder fragment that is only the tail of a query (`` .append(sql` ORDER BY id LIMIT 500`) ``)
is inspected for its `LIMIT` too.

| Option                                                | Type      | Default | Meaning / config errors                                                                                       |
| ----------------------------------------------------- | --------- | ------- | ------------------------------------------------------------------------------------------------------------- |
| `shapeOptions.literalLimit.allowedValues`             | integer[] | `[1]`   | Literal values allowed in `LIMIT` / `FETCH FIRST`. A negative value is a configuration error.                 |
| `shapeOptions.keysetOnlySweep.nonSelectivePredicates` | string[]  | `[]`    | Conjuncts (compared after SQL token normalization; keywords and unquoted identifiers fold to lower case, while string literals, dollar-quoted strings, and quoted identifiers retain their contents) that do not narrow the walk. An empty string is a configuration error. |
| `shapeOptions.keysetOnlySweep.ignoreTables`           | string[]  | `[]`    | Tables that may be walked whole (small configuration tables). Entries use SQL identifier spelling: unquoted `Orders` folds to `orders`, while `"Orders"` matches only that exact case. An empty string is a configuration error. An unqualified entry also matches the table in any schema, but a dot inside a quoted name belongs to the name: `items` does not match `"work.items"`. |

Invalid with both shapes banned (and `deleted_at IS NULL` configured):

```sql
SELECT id FROM orders WHERE id > $1 ORDER BY id LIMIT 500;                     -- both shapes
SELECT id FROM orders WHERE ($1::uuid IS NULL OR id > $1) AND deleted_at IS NULL ORDER BY id LIMIT $2;  -- keyset-only
WITH c AS (SELECT id FROM invoices ORDER BY id LIMIT 1000 FOR UPDATE SKIP LOCKED)
  DELETE FROM invoices USING c WHERE invoices.id = c.id;                       -- literal-limit and keyset-only
```

Valid:

```sql
SELECT id FROM orders WHERE next_reconcile_at <= now() ORDER BY next_reconcile_at, id LIMIT $1;
SELECT id FROM orders WHERE account_id = $1 AND id > $2 ORDER BY id LIMIT $3;   -- parent id narrows the walk
SELECT id FROM order_reconcile_work_items WHERE lease_expires_at IS NULL ORDER BY id LIMIT $1;
SELECT 1 FROM accounts WHERE id = $1 LIMIT 1;
```

Findings for both shapes are reported at the line of the count (`literal-limit`) or of
the table (`keyset-only-sweep`) and honor the same suppression directives. The facts
are `SqlStatementFileFacts.limit_uses` and `sweeps`; see
[PostgreSQL fact sources](../postgres-facts.md).

## Valid example

```sql
SELECT 1 WHERE EXISTS (
  SELECT 1 FROM topics WHERE id = $1
  UNION ALL
  SELECT 1 FROM topics WHERE id = $1
);
SELECT id FROM accounts a WHERE NOT EXISTS (SELECT 1 FROM bans b WHERE b.account_id = a.id);
SELECT id FROM accounts WHERE id NOT IN (1, 2, 3);
SELECT account_id FROM orders GROUP BY account_id HAVING COUNT(*) > 0;
```

## Counterexample

```sql
SELECT 1 FROM posts
WHERE EXISTS (
  SELECT 1 FROM topics WHERE topics.post_id = posts.id
  UNION ALL
  SELECT 1 FROM tags WHERE tags.post_id = posts.id
);
SELECT id FROM accounts WHERE id NOT IN (SELECT account_id FROM bans);
SELECT COUNT(*) > 0 AS has_orders FROM orders WHERE account_id = $1;
```

## Fix

Rewrite so the set operation is not inside a correlated `EXISTS`: test
`IN (SELECT … FROM (<set-operation>) alias)` or use one restricted subquery.
Replace `NOT IN (SELECT …)` and a `COUNT(*)` compared with 0 or 1 with
`NOT EXISTS (SELECT 1 FROM … WHERE …)` or `EXISTS (SELECT 1 FROM … WHERE …)`.

## Suppression

Use `no-mistakes-disable-next-line postgres-sql-shape-policy` immediately
before the finding, or `no-mistakes-disable-line` on that line; use the file
directive only for an intentional exception.

## Related rules

[`postgres-required-predicates`](postgres-required-predicates.md) requires
relation filters; [`postgres-idempotent-insert`](postgres-idempotent-insert.md)
covers replay-safe INSERT.

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
