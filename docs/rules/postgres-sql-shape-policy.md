<!-- cspell:ignore uuidv -->

# `postgres-sql-shape-policy`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

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
treated as possibly correlated. For complete alternatives, executed facts cover
a matching fragment only when its contributing append occurrence and physical
byte origins also match. Equal SQL produced at another source site remains
checked; non-enumerated calls retain the existing SQL-text and bind matching.
Every complete
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
unless `unanalyzableSql` is `ignore` or `advise`.
An unknown call receiving a mutable builder invalidates its complete SQL snapshots;
when evaluated in an `if` test, conditional test, or logical left operand, it
also clears the affected SQL prefix. Ordinary calls invalidate legacy SQL fields too. See the
[alternative model](../postgres-facts.md#finite-sql-alternatives) and
[mutable-builder recovery](../postgres-builder-mutations.md) for alias and
execution boundaries.

```yaml
rules:
  - rule: postgres-sql-shape-policy
    scope: repository
    options:
      executorNames: []
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
`unanalyzableSql` defaults to `fail` (`fail`, `ignore`, or `advise`;
other values are a configuration error).
`importSpecifier` has no default; `executorNames` defaults to `[query, read, write]` only when `importSpecifier` is configured.

## Banned function calls

See [Banned function calls](postgres-sql-shape-policy-banned-functions.md).

## Correlated EXISTS examples

See [Correlated EXISTS examples](postgres-sql-shape-policy-correlated-exists.md).

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

`unanalyzableSql: advise` reports residual uncertainty in non-failing advisories.
`--include-sql-verdicts` adds [per-subcheck SQL verdicts](../postgres-sql-verdicts.md).

`include` / `exclude` select source files (empty include means all files).
`sqlInclude` defaults to `**/*.sql`. `bannedShapes` defaults to
`[correlated-exists-set-operation]`. Also accepted, and off unless listed:
`not-in-subquery`, `count-for-existence`, `literal-limit`, `keyset-only-sweep`,
`banned-function-call`.
Unknown `bannedShapes` values are a configuration error. `unanalyzableSql` defaults to `fail` (`fail`, `ignore`, or `advise`;
other values are a configuration error). `importSpecifier` has no default. `executorNames` defaults to `[query, read, write]` only when `importSpecifier` is configured.

### Executor configuration

See [Executor configuration](postgres-sql-shape-policy-executors.md).

## Bounded iteration shapes

See [Bounded iteration shapes](postgres-sql-shape-policy-bounded-iteration.md).

## Valid example

```sql
SELECT 1 WHERE EXISTS (
  SELECT 1 FROM topics WHERE id = $1
  UNION ALL
  SELECT 1 FROM topics WHERE id = $1
);
```

```sql
SELECT id FROM accounts a WHERE NOT EXISTS (SELECT 1 FROM bans b WHERE b.account_id = a.id);
```

```sql
SELECT id FROM accounts WHERE id NOT IN (1, 2, 3);
```

```sql
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
```

```sql
SELECT id FROM accounts WHERE id NOT IN (SELECT account_id FROM bans);
```

```sql
SELECT COUNT(*) > 0 AS has_orders FROM orders WHERE account_id = $1;
```

## Fix

Rewrite so the set operation is not inside a correlated `EXISTS`: test
`IN (SELECT … FROM (<set-operation>) alias)` or use one restricted subquery.
Replace `NOT IN (SELECT …)` and a `COUNT(*)` compared with 0 or 1 with
`NOT EXISTS (SELECT 1 FROM … WHERE …)` or `EXISTS (SELECT 1 FROM … WHERE …)`.

## Analysis

Raw SQL parse failures retain each failing statement's first code token. Separate
statements produce separate findings, and a `no-mistakes-disable-next-line`
directive immediately above one statement suppresses that failure independently.
Successfully parsed sibling statements remain analyzable.

### Effort

Runs at `max` only. Configuring `effort: low` (rule option, repository `effort`, or the invocation override) is not an error: the rule has no low implementation, so the check warns `requested effort low; effective effort max because this rule has no low implementation` and runs at `max`. At max it evaluates executor calls in TS/JS (helpers, builders, and up to 16 finite SQL alternatives per call) and SQL files selected by `sqlInclude`.

### Unanalyzable inputs

Following recall over precision ([how analysis works](../../README.md#how-analysis-works)), inputs this rule cannot prove are never treated as clean; they follow `options.unanalyzable`.

- Inputs this rule cannot prove: executor calls whose SQL has no complete recovered alternative or exceeds the 16-alternative cap (for example `query(buildSql(filters))` with an unrecoverable helper), and SQL, embedded or in a `sqlInclude` file, that fails to parse. They follow `options.unanalyzable`:
  - `report` (default): fails as a finding, e.g. `executed SQL is not statically recoverable for shape policy` or `SQL could not be analyzed for shape policy`.
  - `advise`: a non-failing advisory with the same reason.
  - `ignore`: omitted.

  Definite violations remain findings under every policy. The legacy `unanalyzableSql: fail` spelling means `report`; `advise` and `ignore` keep their names.

The finding target is taken from the enabled `bannedShapes`.

## Suppression

Use `no-mistakes-disable-next-line postgres-sql-shape-policy` immediately
before the finding, or `no-mistakes-disable-line` on that line; use the file
directive only for an intentional exception.

SQL recovered from an imported helper is reported in the helper's module at the helper's line. A directive in the helper's file suppresses it, and so does a directive in the calling file (file-level, on the executor call, or on the imported-helper call). See [Helper-line attribution and suppression](../postgres-facts.md#helper-line-attribution-and-suppression).

## Related rules

[`postgres-required-predicates`](postgres-required-predicates.md) requires
relation filters; [`postgres-idempotent-insert`](postgres-idempotent-insert.md)
covers replay-safe INSERT.

See [additional shape coverage](postgres-sql-shape-policy-coverage.md).
