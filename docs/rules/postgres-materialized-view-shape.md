# `postgres-materialized-view-shape`

<!-- cspell:ignore uuidv localtimestamp nonpartial refreshable -->

A catalog rule for refreshable, indexed materialized views
whose definitions constrain growing base tables.

```yaml
rules:
  - rule: postgres-materialized-view-shape
    scope: repository
    severity: advisory
    options:
      schemaCatalogPath: db/schema.json
      minAdditionalIndexes: 1
      windowFunctions: [fn_min_uuidv7]
      allowBaseTables:
        - { table: countries, reason: catalogue-sized }
```

## Why and when

Concurrent refresh needs a unique index. Ranking reads need their own access
path, and a materialized view over growing tables should not accumulate an
unbounded history. Enable this rule when reviewing schema catalog changes.
Refresh scheduling and per-view EXPLAIN checks are outside its scope.

## What it catches/requires

Only catalog `views` entries with `materialized: true` are checked. Each needs
a unique index suitable for concurrent refresh and at least
`minAdditionalIndexes` other indexes. A partial or expression unique index
does not establish the concurrent-refresh requirement. The refresh index must also be valid, ready, live, and immediate.

Every base table is presumed growing unless `allowBaseTables` names it with
a reason. The defining query must constrain each read through either a range
on an indexed column or a comparison on the leading column of a primary or unique key
using a configured `windowFunctions` function (including equality). A predicate
on a bounded driver also bounds a lookup joined through its full primary key
or a nonpartial unique key, using the key proof shared with
`postgres-bounded-statements`. Every column of a composite lookup key must be
equated to bounded inputs or scalar constants in every predicate alternative,
with at least one key column supplied by a bounded driver. Constants can
complete a composite lookup key; they do not establish a window alone.
A bounded CTE or
derived SELECT can also drive these lookups; aggregate and LIMIT output caps
do not establish bounded base-table reads. An incomplete key,
a join without a unique key, or an unbounded driver still leaves the lookup
unbounded. An ordinary equality, a limit, or
an arbitrary function name does not establish a window. Plain time ranges may
use `now`, `current_timestamp`, `current_date`, or `localtimestamp`. Definitions use the existing SQL view parser and
required-predicate analysis; the request catalog shares parsed definitions with other view checks. Unsupported definitions are reported for review.

Range proofs require the leading column of a valid, ready, live, nonpartial
B-tree index (primary and unique constraints also establish their backing
index). The bound belongs in the SELECT that reads the base table; predicates
on an outer CTE or referenced view are not pushed into its definition. Flatten
parenthesized joins into explicit joins with base-table WHERE bounds when a
definition needs review.

A qualified column of a single-row `VALUES` CTE or a constant `SELECT` CTE
without recursion can supply the same window as its inline scalar expression.
CTE column names
and aliases are respected, including `column1` for unnamed `VALUES` outputs.
Multiple-row CTEs, relation-reading CTEs, subquery-valued bounds, and arbitrary
functions do not establish constant window values.

Supported `FROM` sources are plain tables, catalog views, CTE references,
and derived `SELECT` subqueries, with ordinary alias names and explicit joins.
Sampling, version clauses, hints, and other source extensions require review.
Column-list aliases on named tables, catalog views, or CTE references also
require review because they can change which catalog column a predicate names.
Derived `SELECT` column-list aliases remain supported: they do not change the
inner base-table proof. Function calls in `FROM`, `UNNEST`, and other unsupported
`FROM` factors produce a definition-review finding, even when their arguments
are constants.
Subqueries inside function arguments may read growing tables that the source
collector cannot establish as bounded. `windowFunctions` configures predicate
functions; it does not exempt a function used as a `FROM` source.

Every nested query must be visited by the shared source collector. A subquery
hidden in a location such as `DISTINCT ON` or a named `WINDOW` clause requires
review; ordinary expressions in those clauses remain supported. Move such a
subquery into an explicit CTE or derived `SELECT`, with bounds at its base-table
reads, when that rewrite preserves the query's semantics.

Findings identify the materialized view and the missing index or unbounded
base table. A read inside a referenced view includes its path, for example
`unbounded base table records via view some_view`. A direct unbounded read
has no `via view` suffix. Separate direct and referenced reads remain separate
findings, even when they read the same table. Catalog scope filters apply to
`schemaCatalogPath`.

## Options and defaults

| Option                 | Default  | Meaning                                                                                                                                          |
| ---------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| `schemaCatalogPath`    | Required | Catalog generated by [`postgres catalog`](../cli/postgres.md). Inherited when a selected project declares it.                                    |
| `minAdditionalIndexes` | `1`      | Nonnegative integer count beyond the refresh unique index.                                                                                       |
| `windowFunctions`      | `[]`     | PostgreSQL function names accepted in key predicates, including equality. Match schema qualification and quote mixed-case or dotted identifiers. |
| `allowBaseTables`      | `[]`     | `{table, reason}` entries for bounded-size base tables. Reasons must be nonempty.                                                                |
| `allow`                | `[]`     | Standard `{object, reason}` catalog finding exceptions.                                                                                          |

The rule uses the shared application `severity`: the default `error` fails
`check` when findings remain. The example above sets `severity: advisory` so
findings appear in `advisories` without failing the check. Remove that setting
or use `severity: error` to enforce the rule. With the rule absent or disabled
it performs no analysis. If the rule has multiple applications, the shared
[severity policy](../configuration/rules.md#severity) applies: any error
application keeps the rule failing.

## SQL examples

A compliant migration creates the bounded view and both access paths:

```sql
CREATE MATERIALIZED VIEW recent_orders AS
SELECT id, score FROM orders
WHERE id >= fn_min_uuidv7(now() - interval '7 days');
CREATE UNIQUE INDEX recent_orders_id ON recent_orders (id);
CREATE INDEX recent_orders_score ON recent_orders (score DESC);
```

A bounded driver may read lookup rows by their complete keys, with a window
supplied by a one-row CTE (`users.id` is its primary key):

```sql
CREATE MATERIALIZED VIEW recent_authors AS
WITH w AS (VALUES (fn_min_uuidv7(now() - '7 days'::interval)))
SELECT p.id, u.name FROM posts p
JOIN w ON p.id >= w.column1
JOIN users u ON u.id = p.author_id;
CREATE UNIQUE INDEX recent_authors_id ON recent_authors (id);
CREATE INDEX recent_authors_name ON recent_authors (name);
```

A timestamp range may also use `WITH w(days) AS (VALUES (7))` and
`published_at > CURRENT_TIMESTAMP - w.days * INTERVAL '1 day'`, provided
`published_at` leads an eligible index.

The counterexample migration has no indexes or base-table window:

```sql
CREATE MATERIALIZED VIEW all_orders AS SELECT id, score FROM orders;
```

## Valid example

Compliant example: the generated catalog records the window and both indexes.
Configure `windowFunctions: [fn_min_uuidv7]` and retain `orders.id` as a
primary or unique key. A plain range on an indexed timestamp column is another
supported window.

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "columns": { "id": { "dataType": "uuid" }, "score": { "dataType": "integer" } },
      "primaryKey": { "columns": ["id"] }
    }
  },
  "views": {
    "recent_orders": {
      "materialized": true,
      "definition": "SELECT id, score FROM orders WHERE id >= fn_min_uuidv7(now() - interval '7 days')",
      "indexes": {
        "recent_orders_id": {
          "unique": true,
          "valid": true,
          "ready": true,
          "live": true,
          "accessMethod": "btree",
          "keys": [{ "column": "id", "expression": "id" }]
        },
        "recent_orders_score": {
          "valid": true,
          "ready": true,
          "live": true,
          "accessMethod": "btree",
          "keys": [{ "column": "score", "expression": "score" }]
        }
      }
    }
  }
}
```

## Counterexample

```json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "columns": { "id": { "dataType": "uuid" }, "score": { "dataType": "integer" } },
      "primaryKey": { "columns": ["id"] }
    }
  },
  "views": {
    "all_orders": {
      "materialized": true,
      "definition": "SELECT id, score FROM orders",
      "indexes": {}
    }
  }
}
```

This has no refresh index, no ranking index, and no bound on `orders`.
Whole-table aggregates such as `SELECT count(*) FROM orders` still read
unbounded input. For a primary key `(tenant_id, id)`, a window on `id` alone
also remains unbounded because it does not lead that key's index. Joining a
lookup on only `id` does not complete the composite key either.

## Fix

Add a nonpartial, column-only unique index for concurrent refresh and the
configured number of additional read indexes. Add an indexed range predicate
or a configured key-window function to every growing base-table read. Use a
reasoned base-table allowance only for tables whose size is bounded by their
domain contract, then regenerate the catalog.

For an unsupported `FROM` function, expose its base-table reads in a bounded
CTE or derived `SELECT`, with the window at each actual read. Move a
set-returning call into the `SELECT` projection only when that rewrite
preserves the query's results, row multiplicity, and correlation. If no
supported equivalent preserves the semantics, allow the materialized view
under `options.allow` with a reason. A base-table allowance does not suppress
an unsupported `FROM` factor. For a named-table or CTE column-list alias,
move column renaming into the `SELECT` projection and use catalog column names
in predicates. Replace sampling or source extensions with an equivalent
supported query only when its semantics are preserved; otherwise record a
reasoned whole-view allowance.

## Analysis

### Effort

Runs at `max` only. A requested `low` effort falls back to `max` with the
standard warning. The rule reads only the selected schema catalog and does
not collect executor facts from TS/JS.

### Unanalyzable inputs

A materialized view without an `indexes` key has unknown index metadata,
as in older format-version-2 snapshots. The rule emits one catalog finding
asking you to regenerate with `no-mistakes postgres catalog --coverage complete`,
and skips index requirements for those views. It still checks their definitions.
An explicit `"indexes": {}` records that there are no indexes and produces the
usual unique-index and additional-index findings. Regeneration requires the
connection and output arguments documented by [`postgres catalog`](../cli/postgres.md).

A missing, unreadable, or invalid catalog is a configuration error. An ordering-only catalog produces an incomplete-input error; regenerate it with `postgres catalog --coverage complete`. A definition
that the shared SQL parser cannot analyze, an unsupported `TABLE` set-operation
arm, a parenthesized join, or an unsupported `FROM` factor is reported for
review; it does not establish that its base tables are bounded. The rule does not infer application
growth rates or inspect refresh scheduling.

## Suppression

Prefer an explicit catalog exception with a reason:

```yaml
rules:
  - rule: postgres-materialized-view-shape
    severity: advisory
    options:
      schemaCatalogPath: db/schema.json
      allow:
        - object: materialized-view:legacy_totals
          reason: Refresh and access-path redesign tracked separately
```

The standard `no-mistakes-disable-file postgres-materialized-view-shape`
directive is also supported on the catalog file. Catalog findings have no
SQL source line, so line and next-line directives cannot target a definition
embedded in a JSON string. Suppression caveat: allowances document an
exception; they do not prove that the view is bounded or refreshable.

## Node API

Async `check()` and `analyzeProject()` check reports run the same configured
rule and honor the same application `severity`. The package exports
`PostgresMaterializedViewShapeOptions`,
`PostgresMaterializedViewBaseTableAllowance`, and
`PostgresMaterializedViewShapeAllowEntry`. See the [Node API](../node-api.md).

## Related rules

- [`postgres-required-predicates`](postgres-required-predicates.md) checks query predicates.
- [`postgres-bounded-statements`](postgres-bounded-statements.md) checks statement cardinality.
- [`postgres-required-comments`](postgres-required-comments.md) checks catalog object documentation.
