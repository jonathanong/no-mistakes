# `postgres-explicit-columns`

A `projects: [<name>]` entry inherits [`projects.<name>.postgres`](../configuration/projects.md#postgresql-scope) when that block is set. Do not restate `importSpecifier`, `executorNames`, `executorFactoryNames`, `executorTypeNames`, `trustedSqlTags`, `schemaCatalogPath`, or `unanalyzableSql` on the rule. `include`, `exclude`, and `sqlInclude` may only narrow the project scope.

Require an explicit column list instead of `SELECT *` or `RETURNING *` when the
relation is wider than `maxColumns`, or when it is named in `relations`.

```yaml
rules:
  - rule: postgres-explicit-columns
    options:
      include: ["src/**/*.ts", "queries/**/*.sql"]
      exclude: ["**/*.test.ts"]
      importSpecifier: "@example/db"
      sqlInclude: ["queries/**/*.sql"]
      schemaCatalogPath: db/schema.json
      maxColumns: 12
      relations: [orders, accounts]
      allowWholeRowFunctions: [row_to_json, to_json, to_jsonb, json_agg, jsonb_agg]
      checkReturning: true
      unanalyzableSql: ignore
```

## Why and when

`SELECT *` fetches every column, including large values the caller
never reads, and it blocks index-only scans. A column added later is also
returned to every star caller, which is how sensitive columns leak into API
responses. Enable this on application queries that should name the columns they
use. Views stay allowed unless you list them in `relations`, because a view's
column list is itself a reviewed contract.

## What it catches/requires

A standalone `TABLE accounts` reads every column of `accounts`, so it is
checked like `SELECT * FROM accounts` when that relation is configured.
The same check applies when `TABLE accounts` is an arm of a supported set
operation, such as `SELECT id FROM safe UNION ALL TABLE accounts`. This
also covers TABLE arms inside UPDATE and DELETE subqueries; an EXISTS arm
remains exempt from explicit-column stars.

The rule reads star projections from the statement pass and column counts from
the schema catalog. It reports a star when the catalog table has more than
`maxColumns` columns, or when the relation is listed in `relations`.
`maxColumns: 0` reports every catalog table, whatever its width.

It does not report `EXISTS (SELECT *)`, `NOT EXISTS (SELECT *)`, `COUNT(*)`, a
star inside `allowWholeRowFunctions`, or a star over a CTE or derived-table
alias. Recursive WITH makes every sibling alias local in each body; non-recursive
WITH makes an alias local only after its definition. Schema-qualified physical
relations remain checked. The inner `SELECT *` of that CTE or subquery is still checked. One
finding is emitted per star and relation. `RETURNING *` uses the same rule when
`checkReturning` is true, including data-modifying CTEs. Bare RETURNING stars
expand the mutation target; qualified stars can also refer to UPDATE FROM or
DELETE USING tables. Schema-qualified relations retain their full catalog
identity, and table-valued functions are not catalog tables.

With no `relations` entries the rule still reports catalog tables wider than the
default `maxColumns` of 12. `schemaCatalogPath` is required, so enabling the
rule with no options is a config error rather than a silent skip.

Every complete [SQL alternative](../postgres-facts.md#finite-sql-alternatives)
of a branching executor call is checked. A choice between `SELECT id` and
`SELECT *` still reports the star path when its relation is covered; all
explicit-column paths pass without an unanalyzable finding. Recovery is capped
at 16 versions and preserves branch-fragment source locations for suppressions.
Opaque or over-cap calls retain `unanalyzableSql` behavior.

## Options and defaults

`unanalyzableSql: advise` reports residual uncertainty in non-failing advisories.
`--include-sql-verdicts` adds [per-subcheck SQL verdicts](../postgres-sql-verdicts.md).

`include`, `exclude`, `importSpecifier`, `executorNames`, and `unanalyzableSql`
match `postgres-required-predicates`. `unanalyzableSql` defaults to `fail`.

`sqlInclude` defaults to `[]`, so `.sql` files are not scanned until you set
globs. `schemaCatalogPath` has no default, is required, and names a catalog generated with [`no-mistakes postgres catalog`](../cli/postgres.md). `maxColumns` defaults
to `12`; `0` means every catalog table. Values outside `0` through `4294967295` are configuration errors.
`relations` defaults to `[]`. An empty name or a duplicate name is a config
error. `allowWholeRowFunctions` defaults to `row_to_json`, `to_json`,
`to_jsonb`, `json_agg`, and `jsonb_agg` (matched case-insensitively). An empty
list means no function is exempt. An empty or duplicate name is a config error.
`checkReturning` defaults to `true`.

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
A configured module enables `.query` only for proven executor receivers.
Explicit `executorNames: [query]` without a module enables broad member matching.
A configured module also recognizes
its transaction helpers. Native SQL and recovered SQL-builder fragments retain
their existing scopes. See [the migration notes](../migrations/explicit-postgres-executors.md).

## Valid example

```jsonc
// db/schema.json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "columns": {
        "id": {
          "dataType": "uuid"
        },
        "status": {
          "dataType": "text"
        },
        "account_id": {
          "dataType": "uuid"
        }
      }
    },
    "tags": {
      "columns": {
        "id": {
          "dataType": "uuid"
        },
        "name": {
          "dataType": "text"
        },
        "slug": {
          "dataType": "text"
        }
      }
    },
    "accounts": {
      "columns": {
        "id": {
          "dataType": "uuid"
        },
        "name": {
          "dataType": "text"
        }
      }
    }
  }
}
```

```sql
-- queries/examples.sql
SELECT id, status FROM orders WHERE id = $1;
SELECT * FROM tags;
SELECT EXISTS (SELECT * FROM orders WHERE account_id = $1);
SELECT COUNT(*) FROM orders;
SELECT row_to_json(o.*) FROM orders o WHERE id = $1;
WITH r AS (SELECT id, status FROM orders) SELECT * FROM r;
```

`tags` has three columns, under the default limit of 12. `orders` is listed
in `relations`, so the explicit list and the `row_to_json` wrapper pass even
though its catalog lists only three columns.

## Counterexample

```jsonc
// db/schema.json
{
  "formatVersion": 2,
  "coverage": "complete",
  "tables": {
    "orders": {
      "columns": {
        "id": {
          "dataType": "uuid"
        },
        "status": {
          "dataType": "text"
        },
        "account_id": {
          "dataType": "uuid"
        }
      }
    },
    "tags": {
      "columns": {
        "id": {
          "dataType": "uuid"
        },
        "name": {
          "dataType": "text"
        },
        "slug": {
          "dataType": "text"
        }
      }
    },
    "accounts": {
      "columns": {
        "id": {
          "dataType": "uuid"
        },
        "name": {
          "dataType": "text"
        }
      }
    }
  }
}
```

```sql
-- queries/examples.sql
SELECT * FROM orders WHERE id = $1;
SELECT o.*, a.name FROM orders o JOIN accounts a ON a.id = o.account_id;
UPDATE orders SET status = 'paid' WHERE id = $1 RETURNING *;
WITH recent AS (SELECT * FROM orders WHERE id > $1) SELECT id FROM recent;
```

`orders` is listed in `relations`. The join reports `orders`. It also reports `accounts`
when that name is in `relations` and the query stars it. The CTE reports the
inner `SELECT *`.

When a local or directly imported helper returns a static SQL template, a
finding points to the star in that helper's source file and literal. Assignment
to a local variable and static `.append()` fragments retain the file and line
of each original literal, including separate stars in a composed query. An
imported SQL tag such as `sql-template-strings` is supported inside these
helpers. The caller must be included in the check; an unused exported helper
does not create a finding. Imported helper recovery follows the shared rules in
[PostgreSQL fact sources](../postgres-facts.md#imported-and-literal-identifier-helpers):
relative, `tsconfig` `paths` and barrel imports, and calls whose arguments are
all string literals. Non-literal arguments, unresolved or ambiguous imports,
and computed helper calls remain unanalyzable.

## Fix

List the columns the caller uses:

```sql
SELECT id, status FROM orders WHERE id = $1;
```

For a whole-row JSON payload, keep the star inside a function listed in
`allowWholeRowFunctions`, such as `row_to_json(o.*)`.

## Analysis

### Effort

Runs at `max` only. Configuring `effort: low` (rule option, repository `effort`, or the invocation override) is not an error: the rule has no low implementation, so the check warns `requested effort low; effective effort max because this rule has no low implementation` and runs at `max`. At max it evaluates executor calls in TS/JS (helpers, builders, and up to 16 finite SQL alternatives per call) and SQL files selected by `sqlInclude`.

### Unanalyzable inputs

Following recall over precision ([how analysis works](../../README.md#how-analysis-works)), inputs this rule cannot prove are never treated as clean; they follow `options.unanalyzable`.

- Inputs this rule cannot prove: executor calls whose SQL has no complete recovered alternative or exceeds the 16-alternative cap (for example `query(buildSql(filters))` with an unrecoverable helper), and SQL, embedded or in a `sqlInclude` file, that fails to parse. They follow `options.unanalyzable`:
  - `report` (default): fails as a finding, e.g. `executed SQL is not statically recoverable for explicit columns` or `SQL could not be analyzed for explicit columns`.
  - `advise`: a non-failing advisory with the same reason.
  - `ignore`: omitted.

  Definite violations remain findings under every policy. The legacy `unanalyzableSql: fail` spelling means `report`; `advise` and `ignore` keep their names.

## Suppression

SQL and TypeScript honor `-- no-mistakes-disable-line`,
`-- no-mistakes-disable-next-line`, and `-- no-mistakes-disable-file` for
`postgres-explicit-columns`. Findings point to each star projection, and
embedded SQL parse failures point to their host call or declaration, so line
suppression applies at the affected query. For a recovered imported helper,
line suppression at the original SQL literal applies to that finding.

SQL recovered from an imported helper is reported in the helper's module at the helper's line. A directive in the helper's file suppresses it, and so does a directive in the calling file (file-level, on the executor call, or on the imported-helper call). See [Helper-line attribution and suppression](../postgres-facts.md#helper-line-attribution-and-suppression).

## Related rules

`postgres-required-predicates` requires predicates on named columns.
`postgres-sql-shape-policy` bans selected query shapes. This rule only cares
about star projections and `RETURNING *`.
