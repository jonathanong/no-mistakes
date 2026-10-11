<!-- cspell:ignore uuidv -->

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
once. Every complete
[SQL alternative](../postgres-facts.md#finite-sql-alternatives) of an executed
branching call is checked, up to 16 versions. A banned shape in one version,
such as `NOT IN (SELECT ...)`, is reported even when its sibling version uses
an allowed shape. Branch-fragment lines retain their suppression locations.
An executor-line suppression also covers findings from the append occurrences
that contributed to that execution. Another append occurrence that reuses the
same SQL constant or fragment remains checked, including when its findings
point to the same physical SQL token. A directive on that shared SQL token's
line suppresses the clause wherever it is reused.
Unparseable SQL or dynamic SQL without complete alternatives, including builder fragments, fails closed
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
          nonSelectivePredicates: ["deleted_at IS NULL"]
          ignoreTables: []
      unanalyzableSql: fail
```

`sqlInclude` defaults to `**/*.sql`. `bannedShapes` defaults to
`[correlated-exists-set-operation]`. `not-in-subquery`, `count-for-existence`,
`literal-limit`, `keyset-only-sweep`, and `banned-function-call` are opt-in.
Numeric digit separators in literal limits are decoded (`LIMIT 1_000` is
`LIMIT 1000`). Unknown `bannedShapes` values are a configuration error.
`unanalyzableSql` defaults to `fail` (`fail` or `ignore`;
other values are a configuration error).
`importSpecifier` has no default; `executorNames` defaults to `[query, read, write]` only when `importSpecifier` is configured.

`banned-function-call` is opt-in. Set a nonempty
`shapeOptions.bannedFunctionCall.functions` list to the SQL function names to
ban. An unqualified configured name matches the final name part, whether the
SQL call is qualified or unqualified: `pg_sleep` matches both `pg_sleep(...)`
and `pg_catalog.pg_sleep(...)`. A qualified configured name must match every
name part. Unquoted SQL and configured identifiers follow PostgreSQL folding
rules; quoted identifiers retain their exact case. Calls are matched from parsed function-expression facts, so
column references and aliases with the same spelling are not reported.

The check covers calls in CTEs, subqueries, mutations, native SQL, configured
executor calls, routine bodies, and statically recovered `EXECUTE` strings.
This is useful in parallel tests: `pg_sleep` occupies a database connection
for the duration of the delay and can stall other work waiting for a pooled
connection.

```yaml
rules:
  - rule: postgres-sql-shape-policy
    scope: repository
    include: ['**/*.test.ts', 'test-helpers/**']
    options:
      importSpecifier: '@example/db'
      trustedSqlTags: [{module: '@example/db', name: sql}]
      bannedShapes: [banned-function-call]
      shapeOptions:
        bannedFunctionCall:
          functions: [pg_sleep, pg_catalog.pg_advisory_lock]
```

Each function entry may also be an object that limits the ban to clauses:

```yaml
shapeOptions:
  bannedFunctionCall:
    functions:
      - pg_sleep
      - name: uuidv7
        clauses: [where, join-on, having]
        hint: 'compare the key against a stable bound computed once per statement'
```

A string entry bans every call and retains the existing diagnostic. An object
without `clauses` also bans every call. The optional `clauses` list accepts
`where`, `join-on`, `having`, `select-list`, `order-by`, `values`, `set`,
`default`, and `returning`. Calls inherit the innermost clause: a subquery's
WHERE overrides an outer SELECT list, and nested function arguments and CASE
expressions keep their containing clause. Column defaults in CREATE TABLE and
ALTER TABLE SET DEFAULT use `default`. ON CONFLICT assignments use `set`, its
arbiter and action WHERE predicates and partial-index WHERE predicates use `where`,
and MERGE WHEN conditions use
`where`. Recovered procedural IF, PERFORM, and RETURN expression wrappers and
window PARTITION BY or frame expressions stay unscoped. Genuine nested SELECT
clauses and window ORDER BY expressions retain their clauses.

Scoped findings name the clause, for example `uuidv7() is banned in WHERE`.
An optional `hint` appends `; <hint>` to the finding. Function names must be
nonempty and unique after PostgreSQL identifier decoding. An empty clause list,
unknown or duplicate clause, or blank hint is a configuration error. This policy
does not infer function volatility from the catalog.

For the scoped example above, `DEFAULT uuidv7()`, `VALUES (uuidv7())`,
`SELECT uuidv7()` and `UPDATE orders SET id = uuidv7()` pass. A predicate such as
`WHERE id >= uuidv7(INTERVAL '-30 days')` fails; compute a stable bound once per
statement and compare against that bound instead. Views, CTEs, routines and
recovered literal EXECUTE commands use the same clause facts.

An empty or omitted `functions` list is a configuration error when
`banned-function-call` is enabled. It has no default effect on the rule's
existing shapes. Unknown shape names remain configuration errors.

Counterexample: a test executor sleeps while holding a database connection.

```ts
await query(sql`SELECT pg_catalog.pg_sleep(30)`);
```

Fix: use a controlled test barrier or fixture synchronization instead of a
time delay. Suppress an intentional call with
`no-mistakes-disable-next-line postgres-sql-shape-policy` or
`no-mistakes-disable-line`; use
`no-mistakes-disable-file` only when the whole file is an intentional
exception.

```sql
SELECT pg_catalog.clock_timestamp();
```

This call remains allowed when only `pg_sleep` is configured.

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
`not-in-subquery`, `count-for-existence`, `literal-limit`, `keyset-only-sweep`,
`banned-function-call`.
Unknown `bannedShapes` values are a configuration error. `unanalyzableSql` defaults to `fail` (`fail` or `ignore`;
other values are a configuration error). `importSpecifier` has no default. `executorNames` defaults to `[query, read, write]` only when `importSpecifier` is configured.

### Executor configuration

Omitting both `importSpecifier` and `executorNames` is a configuration error.
Set `importSpecifier` to your database module or list `executorNames` explicitly.
Use `executorNames: []` without a module to select no executor calls and retain
SQL-file/native-SQL analysis where supported. See the
[executor migration](../migrations/explicit-postgres-executors.md).

| Option            | Default                                                      | Behavior                                                                            |
| ----------------- | ------------------------------------------------------------ | ----------------------------------------------------------------------------------- |
| `importSpecifier` | Empty                                                        | Set explicitly to your database module to match its named imports.                  |
| `executorNames`   | Empty without a module; `[query, read, write]` with a module | Without a module, only explicitly listed names match named imports from any module. |
| `executorFactoryNames` | Empty | Named imports (from `importSpecifier`, a relative path that resolves into that package, or any module when it is empty) whose call result bound with `const`, `let`, `using`, or `await using` is an executor inside the declaring block. |
| `executorTypeNames` | Empty | Imported type names (`import type` or inline `type` specifiers) from `importSpecifier`, a relative path that resolves into that package, or any module when it is empty, whose annotated parameters, including optional and inline-destructured ones, are executors inside the declaring function. |
| `trustedSqlTags` | Empty | Named imports of `name` from `module`, or a subpath of `module`, are parameterized SQL tags. A renamed local binding is trusted. A default import is not. A shadowed or rebound local fails closed. The same name from another module, or a sibling prefix such as `@example/dbx`, stays untrusted. |
| `reportUnmatchedExecutorNames` | `false` | Opt in to one finding per `executorFactoryNames` / `executorTypeNames` entry that none of the files this rule scans imports from `importSpecifier`, its subpaths, or a relative path that resolves into that package (a typo or a wrong module silently disables checking). Reported against the config file, so line suppression does not apply; remove the entry or leave this off to silence it. Rust rule only. See [Reporting unmatched names](../migrations/explicit-postgres-executors.md#reporting-unmatched-names). |

`executorFactoryNames` and `executorTypeNames` add scoped executors: `tx` in
`await using tx = await openTransaction()` or `run` in `run: TxExecutor` is scanned
only inside the block or function that declares it, and a same-named identifier
elsewhere is not. Matching is by configuration only, both default to empty, and
neither counts as executor selection: still set `importSpecifier` or `executorNames`.
With `importSpecifier` set, these two options also match imports from its subpaths
(`@example/db/types` for `@example/db`) but not sibling packages such as
`@example/dbx`. A relative import (`../transaction`) matches only when the resolved
file is inside the package `importSpecifier` resolves to; a same-named import outside
that package does not. If the package root cannot be determined, relative imports do
not match. `executorNames` still requires the exact module.

Omitting both options is a configuration error. Set `executorNames: []` without
`importSpecifier` to explicitly skip executor calls (including `.query`).
A configured module or explicit `query` enables `.query` members. A configured module also recognizes
its transaction helpers. Native SQL and recovered SQL-builder fragments retain
their existing scopes. See [the migration notes](../migrations/explicit-postgres-executors.md).

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

Expanded lexicographic cursors such as `a > $1 OR (a = $1 AND b > $2)` count like tuple cursors when each equality prefix uses the same bind, the keys follow the leading `ORDER BY` columns, and each range arm moves consistently forward or backward through its own sort direction. Mixed sorts such as `ORDER BY a ASC, b DESC` accept `a > $1 OR (a = $1 AND b < $2)`. The final arm may be inclusive (`>=` or `<=`); earlier arms must remain strict because equality there swallows the remaining prefix chain. The OR arms may appear in any order; each must still form exactly one step in the same equality-prefix chain. Either side of each comparison may carry the bind, with the range direction inverted when the bind appears first. Built-in int4 cast spellings, including the exact `pg_catalog.int4` identity, preserve bind identity across equality-prefix arms. Tuple elements normalize recursively; other casts match only when their complete expressions are identical. For `PREPARE`, cast safety follows the referenced parameter declaration: an integer bind stays transparent beside unrelated text parameters, while non-integer or undeclared bind types remain distinct because conversion to int4 can change the bound value. Quoted catalog identifiers retain their case-sensitive meaning, and user-defined type lookalikes remain opaque.

Bind-only guards without column references, calls, or subqueries do not narrow a walk.

`keyset-only-sweep` reports a walk over a whole table by its key: a query with a
`LIMIT` over one base table (no join, grouping or set operation) ordered only by
plain columns of that table, where every top-level `WHERE` conjunct is either a
keyset cursor or a configured non-selective predicate. A cursor compares an `ORDER
BY` column with a bind parameter using `>`, `>=`, `<` or `<=`: `id > $1`,
`(created_at, id) > ($1, $2)`, or the optional form `($1::uuid IS NULL OR id > $1)`.
A single-relation `TABLE orders ORDER BY id LIMIT $1` is also a whole-table page,
with the same finding location and suppression support as its `SELECT * FROM`
equivalent. To narrow the work, rewrite it as a `SELECT` with a selective predicate.
CTE references, non-column order expressions, and zero or NULL caps retain their
existing exclusions.

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
builder fragment that is only the tail of a query (``.append(sql` ORDER BY id LIMIT 500`)``)
is inspected for its `LIMIT` too.

| Option                                                | Type      | Default | Meaning / config errors                                                                                                                                                                                                                                                                                                                                                                |
| ----------------------------------------------------- | --------- | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `shapeOptions.literalLimit.allowedValues`             | integer[] | `[1]`   | Literal values allowed in `LIMIT` / `FETCH FIRST`. A negative value is a configuration error.                                                                                                                                                                                                                                                                                          |
| `shapeOptions.keysetOnlySweep.nonSelectivePredicates` | string[]  | `[]`    | Conjuncts (compared after SQL token normalization; keywords and unquoted identifiers fold to lower case, while string literals, dollar-quoted strings, and quoted identifiers retain their contents) that do not narrow the walk. An empty string is a configuration error.                                                                                                            |
| `shapeOptions.keysetOnlySweep.ignoreTables`           | string[]  | `[]`    | Tables that may be walked whole (small configuration tables). Entries use SQL identifier spelling: unquoted `Orders` folds to `orders`, while `"Orders"` matches only that exact case. An empty string is a configuration error. An unqualified entry also matches the table in any schema, but a dot inside a quoted name belongs to the name: `items` does not match `"work.items"`. |
| `shapeOptions.bannedFunctionCall.functions`            | (string or object)[] | — | Required and nonempty when `banned-function-call` is enabled. Function names follow PostgreSQL identifier case rules; unqualified names match the final part of any qualified call, while qualified names require all parts to match. |

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

For `keyset-only-sweep`, a zero-row page remains empty through parentheses, unary
signs and standard PostgreSQL numeric casts (`LIMIT +0`, `LIMIT 0::bigint`).
Unquoted catalog-qualified numeric casts such as `LIMIT 0::pg_catalog.int8` also
prove an empty page. Application-schema types, quoted custom type spellings, and
casts through nonnumeric intermediate types stay conservative. This semantic check does not change
`literal-limit` classification: cast or unary expressions remain non-literal.
