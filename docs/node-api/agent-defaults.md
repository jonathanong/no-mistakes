# Agent Defaults

- Pass `root` explicitly.
- Omit `tsconfig` to use automatic per-workspace resolution; pass it explicitly
  only to force one config for debugging or compatibility.
- Use `analyzeProject()` when several reports share the same root/config.
  Batch `testsPlan` and `ciTopology` in one `analyzeProject({ reports })` call
  so they share the machine-wide lock. `testsPlan()` / `testsImpact()` return
  camelCase `executionTargets` (optional `name` for Swift path-prefix groups).
  Execution targets retain normalized runner selectors in `runnerArgs`, so
  distinct Cargo `--test` targets and Swift `--filter` values remain separate
  while identical selectors can be combined.
  `includeGlob` is a `testsPlan()` option that scopes configured framework
  discovery before planning, so group accounting and execution targets contain
  only matching tests.
  `ciTopology()` reads the current filesystem for each call; pass `profile: "ci"`
  (or CLI `--profile ci`) to clear command and lock timeouts.
- Prefer structured API results over parsing human CLI output.

`generatePostgresCatalog` is a runtime export. Its named public types are `PostgresCatalogOptions`, `PostgresCatalogCoverage`, `PostgresSearchPathEvidence`, `PostgresCompleteCatalog`, `PostgresOrderingCatalog` and their union `PostgresCatalog`. The overloads return `PostgresOrderingCatalog` for `coverage: "ordering"` and `PostgresCompleteCatalog` otherwise. `searchPathSchemas` opts into exact schema accessibility and relation-name evidence in `searchPathEvidence`; an omitted entry is unknown. Generate evidence with the role that executes the analyzed SQL.

`parsePostgresSql()` also returns typed `PostgresSqlQuery` scope facts for SELECT
statements, including relation and CTE visibility, join participants, conservative
predicate contexts and EXISTS correlation. See [SELECT scope facts](../postgres-source-api.md#select-scope-facts).
Read-only CTE and nested query scope spans include closing function syntax;
slice the original SQL with their half-open UTF-8 byte offsets.
An INSERT-source SELECT in a data-modifying CTE also includes closing function
syntax before the INSERT's conflict or RETURNING clause in both its source and
query scope spans.
Set-operation branch scopes slice exactly their own operand, including its
parenthesis wrapper, and exclude query-level ordering and limiting clauses.
Set-operation branch scopes within that source retain the complete SELECT text too.

`check()` applies `nextjs-redirect-destinations` to recovered static tuple maps
and template destinations, and reports incomplete extraction for partially dynamic returns.

The rule option `trackedRoutesOnly: true` restricts matching pages to the request's
Git index inventory through the same async `check()` API. Untracked pages do not
satisfy destinations; ignored-but-staged and sparse indexed pages remain eligible; the default filesystem mode is unchanged.
See [tracked route configuration](../rules/nextjs-redirect-destinations.md#options).

[Node/N-API guide](../node-api.md)
