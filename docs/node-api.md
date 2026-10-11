# Node/N-API Guide

The `no-mistakes` npm package exposes async functions backed by the same Rust
analysis as the CLI. Use it when an agent or tool needs repeated structured
queries without subprocess overhead.

`playwrightCheck` and `playwrightEdges` honor opt-in
[integration route coverage sources](configuration/integration-route-coverage.md).
`resolveConfig` exposes those declarations as `routeCoverageSources`; the
exported `RouteCoverageSource`, `RouteCoverageHelper`, `RouteCoverageFramework`,
and `RouteCoverageAttribution` types describe the contract.

````js
const {
  analyzeProject,
  dependents,
  importUsages,
  symbols,
  testsPlan,
  validateMermaidMarkdown,
  writePlanningImpactArtifacts,
} = require("no-mistakes");

(async () => {
  const impact = await dependents({
    root: process.cwd(),
    files: ["src/api.mts#handler"],
    tests: [
      "vitest",
      "dotnet",
      "swift",
      "python",
      "go",
      "cargo",
      "rails",
      "php",
      "java",
      "kotlin",
      "elixir",
      "dart",
      "jest",
    ],
  });

  const report = await analyzeProject({
    root: process.cwd(),
    reports: [
      { type: "importUsages", filters: ["src/**"] },
      {
        type: "dependents",
        root: "packages/api",
        tsconfig: "tsconfig.json",
        files: ["src/api.mts#handler"],
      },
      { type: "symbols", files: ["src/api.mts"], include: "both" },
      { type: "symbols", files: ["src/api.mts"], mode: "signature-impact", symbol: "handler" },
      { type: "reactUsages", target: "src/Button.tsx#Button", include: "stories,tests,props" },
      { type: "check", config: ".no-mistakes.yml" },
    ],
  });

  const mermaid = await validateMermaidMarkdown({
    content: "```mermaid\nflowchart LR\n  A --> B\n```",
    file: "docs/design.md",
  });

  console.log({ impact, report, mermaid });
})();
````

PostgreSQL rules invoked through `check()` or an `analyzeProject()` check query
use the same explicit executor configuration as the CLI. Set `importSpecifier`
or `executorNames` in the rule options to select embedded executor calls;
omitting both is a configuration error. Set `executorNames: []` explicitly
without a module to select no executor calls. `executorFactoryNames` and
`executorTypeNames` additionally select block- or function-scoped executors from
factory results and executor-typed parameters, imported from `importSpecifier`,
a subpath of it, or a relative path that resolves into that package. `trustedSqlTags` (default empty) opts in to named imports of
`name` from `module`, or a subpath of `module`, as parameterized SQL tags. A
renamed local binding is trusted. A default import is not, and a shadowed or
rebound local fails closed. Rust rules also accept `reportUnmatchedExecutorNames: true` to report
configured names no scanned file imports; it appears in `check()` results like any
other finding. See the
[executor migration notes](migrations/explicit-postgres-executors.md).

`postgres-require-query-annotation` traces straight-line SQL helpers and callback
forwarding through the same prepared project facts used by `check()` and
`analyzeProject()`. Both APIs share the requested TS/JS parse and keep resolution
and helper-evaluation memoization in memory for the duration of the request.
Its `unanalyzableSql` option defaults to `"report"`; choose
`"ignore"` explicitly to skip opaque leading SQL. Named configuration types
`PostgresRequireQueryAnnotationOptions` and `PostgresUnanalyzableSql` are exported
from the package. Findings retain the executor's source location. See the
[annotation rule](rules/postgres-require-query-annotation.md).

`postgres-sql-shape-policy` also accepts `banned-function-call` and its
`shapeOptions.bannedFunctionCall.functions` list through the same YAML rule
configuration. Entries may be strings for an unrestricted ban or objects with
`name`, optional `clauses`, and optional `hint` for clause-specific diagnostics.
Checks consume shared SQL facts with innermost-clause context;
`parsePostgresSql()` retains its existing source-report shape. See the [SQL shape policy](rules/postgres-sql-shape-policy.md).
Calls in executor SQL use the prepared embedded SQL facts; the existing asynchronous `check()` API handles the query, so this option adds no
Node export or declaration.

`postgres-sql-statement-policy` also supports those executor options through
`check()` and check queries. Its embedded analysis is opt-in; omitting executor
configuration preserves SQL-file-only behavior. Rule application `include` and
`exclude` scope executor files; `sqlInclude` still selects SQL files. See the
[statement policy](rules/postgres-sql-statement-policy.md) for test-helper DDL
and `unanalyzableSql` examples. Its `bannedStatements` accepts the `ddl` group
and the rule's documented statement kinds; `bannedSettings` checks configured
static PostgreSQL setting names in `SET`, `set_config()`, and database/system
`SET` statements. These options use the same asynchronous `check()` API and
rule configuration, with no additional Node export.

## PostgreSQL catalog generation

The `postgres-key-column-types` catalog rule is available through the existing
async `check()` API and `analyzeProject({ reports: [{ type: "check" }] })`.
Configure `schemaCatalogPath` and a nonempty `allowedTypes` list in the rule's
options; `checkPrimaryKeys`, `checkForeignKeys`, and `allowEnumTypes` are also
available there. Suppress an intentional exception with a reasoned `allow`
entry targeting the exact `constraint:<table>.<constraint>` identity. The
catalog schema types are exported from `no-mistakes` for typed callers.

For SQL text without a database or repository, use async
[`parsePostgresSql(source)`](postgres-source-api.md), including typed INSERT/ON CONFLICT
expression arbiters (including operator classes and typed parameters), mixed array/composite assignment
targets, source forms, predicates, assignment provenance (including `derived`
functions independent of syntax completeness), typed unary operands, and typed `createView`
facts for `CREATE [OR REPLACE] RECURSIVE VIEW`. A source array returns
facts in input order. Expression roots and ordered direct call arguments are
typed, including cast/parenthesis wrappers and bare SQL value functions. DO bodies expose typed IF/ELSIF/ELSE branch conditions
and nested DDL source occurrences, without claiming that any branch executes.

Expression roots distinguish `nullTest` (`negated` for IS NOT NULL),
`distinctness` (`negated` for IS NOT DISTINCT FROM), `parameter` (`placeholder`,
including `$1`), and `typedLiteral` (`dataType`, decoded string `value`, rendered
`sql`). Recursive children retain `nullOperand` and ordered `distinctLeft` /
`distinctRight` operands under operators, calls, CASE, and wrappers. Temporal
literals retain their type, precision, timezone qualifier, and value: `now`,
`today`, and `epoch` are syntax facts, without a volatility or replay policy.
`childrenComplete` describes represented syntax independently of source spans.
AND/OR trees, null tests, and qualified casts are complete when every operand is
represented. A null span still means that its exact source boundary is unknown;
it does not erase complete structure. Unsupported expressions, omitted children,
and depth limits remain explicitly incomplete and callers must fail closed.
Ordinary `literal` roots also expose `value: PostgresSqlLiteralValue`,
distinguishing SQL null, string, number, and boolean values. Number values retain their decimal spelling as
strings; quoted and escaped string values use the parser-decoded contents.
Unclassified values expose `other` with SQL rather than guessed semantics.

Expression `children` expose ordered typed operands and descendants, with
`childrenComplete` independent of INSERT source lineage. INSERT `columnSources`
maps explicit target columns positionally across VALUES rows and SELECT/set
branches; unsupported or ambiguous forms return a typed reason instead of a
partial mapping. Top-level INSERT facts include `returning`, an array of
`expression`, `wildcard`, or `unsupported` items, empty when the statement has
no RETURNING clause. A data-modifying CTE insert includes `columnSources` only
when that insert has RETURNING, and that lineage matches the direct INSERT of
the same source syntax. Child/source spans are nullable when prepared tokens cannot
prove complete wrapper boundaries. Inspect nullable spans separately from
`childrenComplete`, which describes the full represented syntax.
CREATE INDEX facts retain PostgreSQL's `ON ONLY relation` modifier as
`index.only` in the async API and declarations.
This pure source API accepts no invocation-lock options.

Typed comment metadata preserves routine signatures, and ALTER INDEX metadata
preserves attach-partition and rename operations. Plain newline-separated
string constants are joined while source spans retain their original spelling.

`generatePostgresCatalog({ connectionEnv, schema, coverage, searchPathSchemas, currentDatabase })` asynchronously
returns the schema catalog that `schemaCatalogPath` reads, generated from a live
PostgreSQL schema. `coverage` is `"complete"` (the default; every catalog rule
accepts it) or `"ordering"` (only conflict and lock ordering accept it). It requires
`psql` and keeps connection secrets in the named environment variable. The caller
writes the returned object to disk. See [`postgres catalog`](cli/postgres.md) for
what a catalog holds, what it leaves out, and its limitations.
Both catalog types expose optional `currentDatabase`, the exact connected
database name; older catalogs omit it and preserve conservative analysis. The
optional `currentDatabase` option, like the CLI's `--current-database`, records a
fixed non-empty name instead, so regeneration does not depend on the generating
database's name.

## CLI Mapping

| CLI                                        | Node API                                                                                                                                                                                                                                                                   |
| ------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `dependencies`                             | `dependencies(options)`                                                                                                                                                                                                                                                    |
| `dependents`                               | `dependents(options)`                                                                                                                                                                                                                                                      |
| `related`                                  | `related(options)`                                                                                                                                                                                                                                                         |
| `symbols`                                  | `symbols(options)`                                                                                                                                                                                                                                                         |
| `import-usages`                            | `importUsages(options)`                                                                                                                                                                                                                                                    |
| `importers`                                | `importers(options)`                                                                                                                                                                                                                                                       |
| `exports-of`                               | `exportsOf(options)`                                                                                                                                                                                                                                                       |
| `dead-exports`                             | `deadExports(options)`                                                                                                                                                                                                                                                     |
| `call-sites`                               | `callSites(options)`                                                                                                                                                                                                                                                       |
| `resolve-check`                            | `resolveCheck(options)`                                                                                                                                                                                                                                                    |
| `fetches`                                  | `fetches(options)`                                                                                                                                                                                                                                                         |
| `postgres catalog` | `generatePostgresCatalog(options)` |
| Source-only library capability | `parsePostgresSql(sourceOrSources)` |
| Selected TS/JS modules | `analyzeTypeScriptModules({ root, files })` |
| `flow`                                     | `flow(options)`                                                                                                                                                                                                                                                            |
| `check`                                    | `check(options)`                                                                                                                                                                                                                                                           |
| `config resolve`                           | `resolveConfig(options)`                                                                                                                                                                                                                                                   |
| `data-pw`                                  | `dataPw(options)`                                                                                                                                                                                                                                                          |
| `effects`                                  | `effects(options)`                                                                                                                                                                                                                                                         |
| `rsc-callers`                              | `rscCallers(options)`                                                                                                                                                                                                                                                      |
| `registry-extension`                       | `registryExtension(options)`                                                                                                                                                                                                                                               |
| `tests plan`                               | `testsPlan(options)`; `framework` accepts `vitest`, `playwright`, `dotnet`, `swift`, `python`, `go`, `cargo`, `rails`, `php`, `java`, `kotlin`, `elixir`, `dart`, or `jest`. Import `TestPlanFramework` for that union instead of indexing `TestExecutionTarget['runner']` |
| `tests targets`                            | `testsTargets(options)`                                                                                                                                                                                                                                                    |
| `tests impact`                             | `testsImpact(options)`                                                                                                                                                                                                                                                     |
| `tests why`                                | `testsWhy(options)`                                                                                                                                                                                                                                                        |
| `tests comment`                            | `testsComment(options)`                                                                                                                                                                                                                                                    |
| `tests graph`                              | `testsGraph(options)` or `testsGraphMermaid(options)`                                                                                                                                                                                                                      |
| `playwright check\|edges\|related\|tests`  | `playwrightCheck`, `playwrightEdges`, `playwrightRelated`, `playwrightTests`                                                                                                                                                                                               |
| `queues edges\|related\|check`             | `queueEdges`, `queueRelated`, `queueCheck`                                                                                                                                                                                                                                 |
| `server routes\|edges\|related\|contracts` | `serverRouteList`, `serverRouteEdges`, `serverRouteRelated`, `serverContracts`                                                                                                                                                                                             |
| `react analyze\|check\|usages`             | `reactAnalyze`, `reactCheck`, `reactUsages`                                                                                                                                                                                                                                |
| `infra resource-refs\|outputs\|test-for`   | `infraResourceRefs`, `infraOutputs`, `infraTestFor`                                                                                                                                                                                                                        |
| `swift importers\|test-targets`            | `swiftImporters`, `swiftTestTargets`                                                                                                                                                                                                                                       |
| `lockfile diff`                            | `lockfileDiff(options)`                                                                                                                                                                                                                                                    |
| `ci impact`                                | `ciImpact(options)`                                                                                                                                                                                                                                                        |
| `ci env`                                   | `ciEnv(options)`                                                                                                                                                                                                                                                           |
| `ci topology`                              | `ciTopology(options)`                                                                                                                                                                                                                                                      |
| `ci topology-impact`                       | `ciTopologyImpact(options)`                                                                                                                                                                                                                                                |
| `impacted-checks`                          | `impactedChecks(options)`                                                                                                                                                                                                                                                  |
| `planning-impact` (npm package only)       | `writePlanningImpactArtifacts(options)`                                                                                                                                                                                                                                   |

The `dependencies`, `dependents`, and `related` options accept
`relationships: ["call"]` to select the opt-in lexical call edges. Calls are
resolved only for local functions, direct named imports, static namespace-member
imports such as `api.run()` from `import * as api`, and explicit named re-exports.
`depth: 1` selects direct calls and `depth: 0` selects none;
computed/dynamic callees, globals, and ambiguous re-exports are omitted rather
than guessed. The default relationship set and `"all"` intentionally exclude
call edges.

`writePlanningImpactArtifacts(options)` is an integration helper for callers
that need private, CLI-compatible planning files without spawning several
commands. It runs one prepared `analyzeProject()` request, then writes
`dependencies`, `dependents`, `symbols`, and Vitest `plan` artifacts as JSON,
stderr, and status files in `outputDirectory`. `changedFilesManifest` is the
path to a private regular file directly in that same existing `0700` directory
(exactly `0700`, with no additional permission or special bits); its contents
are newline-delimited literal repository-relative paths (only empty records are
ignored) and its filename cannot collide with an artifact destination. The
directory must remain the same private directory for the duration of the
operation. It must share its parent's filesystem (mount boundaries are
rejected), and that parent must permit atomically renaming the directory:
publication parks and restores it. Bind mounts or other directories the
platform cannot rename are unsupported and fail during that atomic parking.
Artifact files use mode `0600`;
all four existing statuses are first made nonzero before analysis begins, and
failures remove stale JSON and attempt to write bounded diagnostics and nonzero
statuses for every report. Those failure artifacts are best effort: inability
to publish them never masks the original analysis or publication error. If that
reporting transition cannot restore the output directory, the original error's
non-enumerable `failureReportingError` contains the parked recovery path; a
non-extensible thrown value is instead preserved in an `AggregateError`. The
helper serializes concurrent calls that resolve to the same output directory,
covering invalidation, analysis, and publication so an older call cannot mark
its artifacts successful while a newer analysis is pending. Serialization uses
an OS advisory lock shared by Node processes and worker isolates. Its adjacent
`.<output-name>.planning-impact.lock` file is private and empty; it stores no
analysis state, and process or worker-isolate exit releases the lock. Real paths
and symbolic-link aliases use the same canonical lock identity. Changed-files
manifests must be valid UTF-8; a leading UTF-8 BOM is ignored. Safe manifest
resolution failures also replace stale success artifacts with failure output.
The helper is unavailable on Windows because
Node does not expose a trustworthy private Windows ACL check. Structural paths
are sent to traversal reports as `{ file }` entries, so `#` remains part of a
literal filename rather than a symbol delimiter. Deleted or renamed-away
structural paths remain in traversal and test-planning inputs, but are omitted
from the symbols input because no source file remains to analyze.

```js
await writePlanningImpactArtifacts({
  root: process.cwd(),
  changedFilesManifest: "/private/run/changed-files.txt",
  outputDirectory: "/private/run",
  broad: false,
});
```

The npm package also exposes this integration as `no-mistakes planning-impact
--changed-files <manifest> --output-dir <directory>`. Its current working
directory is `root`; `--broad`, `--timeout`, `--lock-timeout`,
`--fail-on-lock`, `--jobs`, and `--profile ci` map to the matching API options.
The command is silent on success. Argument and analysis failures write a
UTF-8-safe diagnostic of at most 4 KiB to stderr and exit `1`. It is
intentionally unavailable from the Cargo-installed native binary.

The following inventory is the complete runtime export surface. Keeping this
list exhaustive makes a newly added function visible to agents even when it
does not have a one-to-one CLI command:

| Runtime export | API |
| --- | --- |
| `createWorkflowTopologyIndex` | `createWorkflowTopologyIndex(topology)` |
| `version` | `version()` |
| `analyzeProject` | `analyzeProject(options)` |
| `writePlanningImpactArtifacts` | `writePlanningImpactArtifacts(options)` |
| `callSites` | `callSites(options)` |
| `check` | `check(options)` |
| `ciEnv` | `ciEnv(options)` |
| `ciImpact` | `ciImpact(options)` |
| `ciTopology` | `ciTopology(options)` |
| `ciTopologyImpact` | `ciTopologyImpact(options)` |
| `dataPw` | `dataPw(options)` |
| `deadExports` | `deadExports(options)` |
| `dependencies` | `dependencies(options)` |
| `dependents` | `dependents(options)` |
| `effects` | `effects(options)` |
| `exportsOf` | `exportsOf(options)` |
| `generatePostgresCatalog` | `generatePostgresCatalog(options)` |
| `parsePostgresSql` | `parsePostgresSql(sourceOrSources)` |
| `analyzeTypeScriptModules` | `analyzeTypeScriptModules({ root, files })` |
| `fetches` | `fetches(options)` |
| `flow` | `flow(options)` |
| `impactedChecks` | `impactedChecks(options)` |
| `importUsages` | `importUsages(options)` |
| `importers` | `importers(options)` |
| `infraOutputs` | `infraOutputs(options)` |
| `infraResourceRefs` | `infraResourceRefs(options)` |
| `infraTestFor` | `infraTestFor(options)` |
| `lockfileDiff` | `lockfileDiff(options)` |
| `validateMermaidMarkdown` | `validateMermaidMarkdown(options)` |
| `playwrightCheck` | `playwrightCheck(options)` |
| `playwrightEdges` | `playwrightEdges(options)` |
| `playwrightRelated` | `playwrightRelated(options)` |
| `playwrightTests` | `playwrightTests(options)` |
| `reactAnalyze` | `reactAnalyze(options)` |
| `reactCheck` | `reactCheck(options)` |
| `reactUsages` | `reactUsages(options)` |
| `registryExtension` | `registryExtension(options)` |
| `related` | `related(options)` |
| `resolveCheck` | `resolveCheck(options)` |
| `resolveConfig` | `resolveConfig(options)` |
| `rscCallers` | `rscCallers(options)` |
| `swiftImporters` | `swiftImporters(options)` |
| `swiftTestTargets` | `swiftTestTargets(options)` |
| `symbols` | `symbols(options)` |
| `testsComment` | `testsComment(options)` |
| `testsGraphMermaid` | `testsGraphMermaid(options)` |
| `queueCheck` | `queueCheck(options)` |
| `queueEdges` | `queueEdges(options)` |
| `queueRelated` | `queueRelated(options)` |
| `queues` | `queues(options)` |
| `serverContracts` | `serverContracts(options)` |
| `serverRouteEdges` | `serverRouteEdges(options)` |
| `serverRouteList` | `serverRouteList(options)` |
| `serverRouteRelated` | `serverRouteRelated(options)` |
| `serverRoutes` | `serverRoutes(options)`; Remix file-based routes appear when a `type: remix` project is configured |
| `testsGraph` | `testsGraph(options)` |
| `testsImpact` | `testsImpact(options)` |
| `testsPlan` | `testsPlan(options)` |
| `testsTargets` | `testsTargets(options)` |
| `testsWhy` | `testsWhy(options)` |

`testsTargets()` and test-plan targets set `workspace: true` when a Vitest
workspace/project-array source must be passed with `--workspace`; the emitted
`runnerArgs` already contain the correct flag. This includes configured and
default-discovered `vitest.workspace.*` and `vitest.projects.*` sources,
including JSON project arrays, matching the CLI. A default-discovered root
workspace/project-array source takes precedence over sibling
`vitest.config.*`; explicitly configured paths remain authoritative.

The Playwright APIs load the same selector-wrapper configuration as the CLI.
Configured wrapper calls therefore appear in `playwrightEdges()` and
`analyzeProject()` through the existing selector-edge JSON shape; no separate
Node option or result type is required.

The graph APIs (`dependencies`, `dependents`, `related`, `flow`, and graph
reports in `analyzeProject`) accept the `workflow`, `workflow-job`,
`workflow-step`, `workflow-needs`, `workflow-uses`, `workflow-run`, and
`workflow-artifact` relationship values. `workflow` includes all six edges;
the precise values retain their required structural job/step bridges for a
connected traversal. `all` includes `workflow`.

Workflow jobs and steps are virtual graph nodes with IDs
`workflow.yml#job:<job>` and `workflow.yml#job:<job>/step:<zero-based-index>`.
`DependencyFile` records expose `workflowFile`, `job`, and optional `step`;
`FlowNode` additionally uses `kind: "workflow-job"` or `"workflow-step"`.
The workflow graph tracks only local, static topology: local reusable workflows
and action descriptors, supported literal `run:` targets/package scripts, and
same-run artifact upload -> download edges. It omits remote `uses`,
`workflow_run`, malformed/dangling endpoints, dynamic shell resolution, and
targets outside the tracked graph universe. `ci` remains the separate legacy
`CiInvocation` relationship from workflow file to supported Rust Cargo binary.

The graph APIs also accept `trpc`. That opt-in relationship follows static
tRPC router procedures and client calls through virtual nodes identified as
`router.ts#procedure:user.get`. `DependencyFile` records expose `routerFile`
and `procedure`; `FlowNode` uses `kind: "trpc-procedure"`. `all` does not
include `trpc`. Empty `projects.*.trpc.routers` lists disable extraction.

`testsPlan(options)` returns `changedFiles`, the sorted, deduplicated
changed-file inventory prepared by that same call, relative to the request root.
The field is present even when no tests are selected and retains deleted paths
plus both sides of detected renames and copies.

Ordinary `testsPlan()` configured `direct` groups select changed tests and
tests one reverse import-family or same-directory `TestOf` edge away from a
changed file. That 1-hop set is selected before `dependencies` and therefore
survives the environment file/percent limit. Markdown, resource, route, and
multi-hop import paths stay in `dependencies`. This is distinct from
`directTestOwner`.

Set `directTestOwner: true` with an explicit `framework` to select only changed
framework-owned tests and framework-owned tests one reverse canonical graph edge
away. This bypasses test-plan environment policy (including groups, limits,
samples, fallback, and include/exclude filtering), attaches normal execution
targets, and returns a `direct-test-owner` group with `fallbackTriggered:
false`. `limitPercent`, `limitFiles`, and `globalConfigFallback` conflict with
this option. `entrypoints` also conflicts with it: direct-owner selection is
bounded to changed files and one reverse canonical graph edge, so use
`testsImpact()` for explicit entrypoint traversal. The returned warnings retain
canonical graph diagnostics for dynamic resource calls in changed files, so
incomplete reverse ownership is visible to API consumers.

The TypeScript declaration models this as a discriminated option: direct-owner
plans require `framework`, while ordinary plans omit `directTestOwner` or set it
to `false`.

`testsPlan(options)` returns `fallbackTriggered` and `fallbackReason` when a
`dotnet` or `swift` plan has to fall back from native graph tracing to
framework-scoped discovered tests. Vitest plans also use this surface for a
dynamic or unresolved `setupFiles`/`globalSetup` declaration: the result is
bounded to its known project owner when possible. Its helper closure follows
ordinary static imports/re-exports and literal CommonJS `require(...)` or
`require.resolve(...)` dependencies, retaining edits and deletions as owner
triggers; computed or non-literal forms are not followed. Resolved setup paths
use `via: ["vitest-setup"]` and may add `viaDetails`, an optional array aligned
with `via` whose setup edge detail is `{ type: "vitest-setup", field:
"setupFiles" | "globalSetup" }`.

`testsWhy()` and `testsGraph()` expose the same optional structured `detail`,
and the Mermaid graph renders the Vitest field in the edge label. The optional
fields preserve compatibility with previously saved plan JSON and are absent
for ordinary edges.

When `testsWhy()` or an `analyzeProject()` report of type `testsWhy` receives
`planJson`, the Node facade materializes that document in a private temporary
directory before calling native code. The directory is removed after success,
native rejection, or preparation failure. Batched reports wait for every plan
to finish preparation before returning an error, so a slower sibling cannot
leave a generated plan directory behind.

`testsImpact()` skips only a failed or unavailable optional Vitest config so a
native test impact remains available. If Vitest configuration prepared
successfully, its discovery errors (such as invalid include patterns) reject
the API call just as they do for direct Vitest discovery.

`testsPlan(options)` rejects (rather than resolving to an empty plan) when
`base`/`head`/`fromGitDiff` can't be resolved by Git — an invalid ref, a
shallow clone missing the merge base, a non-repository root, or a Git exit
failure. The rejection message embeds a stable, greppable diagnostic code
(`git-not-a-repository`, `git-merge-base-unavailable`, `git-shallow-history`,
`git-exit-failure`, `git-malformed-output`) matching the CLI's stderr —
see `docs/cli/tests-plan.md`.

The API uses the same target-scoped `fullSuiteTriggers.projects` behavior as the
CLI. A `{ paths, targets }` match selects only tests owned by those runner
projects, emits `configured-trigger` reasons and execution targets, and leaves
`fallbackTriggered` false. Semantic `.no-mistakes.yml`/`.yaml` invalidation is
also identical for revision and inline-diff inputs.

Structured target triggers do not expand when the matching changed file is a
discovered test unless that trigger sets `includeChangedTests: true`. The
changed test and its graph dependents remain selected normally. Legacy
full-suite trigger forms retain their existing behavior.

`testsPlan`, `testsImpact`, `testsWhy`, and `testsGraph` expose resource-edge
provenance without a separate API: plan reasons use optional edge-aligned
`viaDetails`, why steps use optional `detail`, and graph JSON edges use
optional `detail`. Details are `{ type: "resource", consumerFile,
callSites: [{ callKind, line }] }` for literal runtime filesystem edges or
`{ type: "vitest-setup", field: "setupFiles" | "globalSetup" }` for setup
edges.

`check(options)` returns the same structured check report as CLI JSON,
including configured filesystem rules such as
`package-json-nested-workspace-coverage`, and `warnings: string[]` for checks
that could not run.

PostgreSQL checks share the CLI SQL frontend: parenthesized `TABLE relation`
subqueries retain quoted identity and view cascade dependencies in `check()`
and `analyzeProject()` reports.
Supported `TABLE` set-operation arms also supply exact relation names and source
lines to `postgres-explicit-columns` and `postgres-required-predicates` through
those same async check entrypoints.

It rejects with the same rule application and `options` path diagnostic as the
CLI when a configured option has the wrong type; invalid option objects are
never replaced with rule defaults.

For Swift and C# source rules, `check()` and an `analyzeProject()` `check`
report both ignore comments and string literals while continuing to inspect
executable string interpolations. Finding lines refer to the original source.

`resolveConfig(options)` returns the same JSON as `config resolve`: frontend
apps, Playwright coverage gates, effective per-app `rewrites`/`ignoreRoutes`,
Vitest `vitestFullSuiteTriggers`, and the additive `fullSuiteTriggers` array
keyed by `TestPlanFramework`. Existing `vitestFullSuiteTriggers` contents stay
unchanged.

The Node declarations model the stable report DTOs for `fetches()`, `queues()`,
`reactAnalyze()`, and `check()`. Fetch reports use `FetchOccurrence`,
`DuplicateApiCall`, and `UnsupportedApiCall`; queue reports use typed producer,
worker, job, edge, diagnostic, and check-finding records; React component facts
use typed fetch calls, child references, and inherited aggregate facts. Rust
fields with `skip_serializing_if` are optional in TypeScript and omitted from
JSON when absent; other nullable Rust fields are represented as `string | null`.
Check reports optionally include `suppressed` when `includeSuppressed: true` is
passed. Each `SuppressedFinding` records its domain, rule, source file, reason,
and the matching `file`, `line`, or `nextLine` directive. The report DTOs live in
focused `*-report-types.d.ts` modules and are exported from `no-mistakes` through
the `report-types.d.ts` barrel.

`validateMermaidMarkdown({ content, file? })` validates Mermaid fences in an
in-memory Markdown or MDX document without reading the filesystem. It resolves
asynchronously with `{ valid, diagramCount, diagnostics }`; each diagnostic
identifies the opening `fenceLine` and, when available, Merman's
diagram-relative line, column, and diagram type. With no `file`, clear JSX
component blocks are detected automatically without reinterpreting standard
Markdown HTML blocks. Pass an `.mdx` file name to enable full MDX recovery. Use
the configured `markdown-mermaid-validation` rule when validating tracked
repository files.

Each `analyzeProject()` report may use its report-specific options. Graph
reports may override `root`, `tsconfig`, and `config`; `resolveCheckDependencies`
accepts the same scope overrides, and referenced dependency report IDs are
resolved within that report's effective scope. `reactUsages` accepts
`target`, `targets`, `include`, and scope options; and `check` may override
`root`, `tsconfig`, and `config`. Lightweight queries (`importers`, `exportsOf`,
`deadExports`, `callSites`, `resolveCheck`), `fetches`, test-plan reports,
`lockfileDiff`, CI/infra/swift reports, `impactedChecks`, and
`validateMermaidMarkdown` are also valid `reports[].type` values. They inherit
the request `root`/`tsconfig`/`config` and dispatch through the dedicated Node
APIs. `resolveCheck` import rows include `computed: true` for non-literal
`import()` / `require()` specifiers, which are classified `unresolved`.
`resolveCheck` also resolves recognized workspace package imports through the
same visible `exports`/`main` resolver as dependency `workspace` edges; missing or
blocked workspace subpaths are `unresolved`. Third-party and unmatched packages
remain `external`. The same classification applies to `resolveCheckDependencies`
without rediscovering files or parsing its prepared closure again. Configured
aliases keep precedence, and declaration files satisfy only type imports.
`importUsages` omits those computed rows and keeps string literals, including
expression-free templates such as ``require(`./mod`)``.
Reports with the same effective scope share
one request-scoped in-memory dataset. Sources, parsed metadata, and compact file
facts are reused; each normalized graph or symbol-index plan is built at most
once for its file universe. Distinct effective scopes are prepared independently.

When a request omits `tsconfig`, TypeScript/JavaScript imports are resolved with
the config that owns each importing file. Dependency graph and query APIs plus
test planning use this behavior across referenced workspace projects. Set
`tsconfig` only to force that one config for the entire request; this preserves
the previous single-config behavior for debugging and compatibility.

`compilerOptions.paths` uses TypeScript's selection rules in standalone graph
APIs and `analyzeProject()`: exact keys win, then the longest prefix before a
wildcard, with declaration order breaking ties. Replacement fallbacks are tried
only within the winning key, so a missing specific target cannot resolve through
a less-specific mapping.

`ciTopology(options)` returns the same schema-v1 `WorkflowTopology` JSON as
`ci topology --format json` — it never throws on diagnostics (unlike the CLI,
which exits non-zero and prints nothing when any diagnostic is an error);
callers inspect the returned `diagnostics` array themselves.
`createWorkflowTopologyIndex(topology)` builds a frozen, sorted query index
(`directUpstreamJobIds`, `transitiveCalleeWorkflowPaths`,
`artifactConsumersForProducerJob`, etc.) over that result — it is pure JS,
runs entirely client-side, and never crosses the N-API boundary itself:

Workflow, job, and step nodes also expose authored CI configuration:
environment-variable blocks, static secret-reference names, job runner and
timeout declarations, effective permissions, job outputs, and step
`run`/`with` data. Values are not evaluated. Secret analysis is strictly
name-only and never reads GitHub or process secret material. See
[`ci topology`](cli/ci-topology.md) for the exact schema and normalization
rules.

```js
const { ciTopology, createWorkflowTopologyIndex } = require("no-mistakes");

const topology = await ciTopology({ root: process.cwd() });
const index = createWorkflowTopologyIndex(topology);
index.transitiveDownstreamJobIds(".github/workflows/ci.yml#build");
```

`impactedChecks(options)` shares one in-memory analysis pass across configured
test frameworks. Pass `timings: true` to include an ordered `timings` array in
the report:

```js
const { impactedChecks } = require("no-mistakes");

const report = await impactedChecks({
  root: process.cwd(),
  changedFiles: ["src/api.mts"],
  genericOnly: true,
  timings: true,
});

// report.timings: [{ phase: "prepare", duration_ms: 12 }, ...]
```

Timing entries use stable phase identifiers and fractional-millisecond
durations. The lazy `graph` phase is present only when dependency analysis is
needed. The property is omitted by default. Unlike CLI `--timings`, Node timing
collection does not print progress to stderr.

If no checks are selected, the report includes `empty_result` with a stable
`code` (`no-changed-files` or `no-impacted-checks`) and a human-readable
`message`. It is omitted from reports containing checks. The async API remains
stderr-free, including for empty results.

Set `genericOnly: true` to return only configured `checks.commands` entries.
It preserves changed-file collection but skips test-framework discovery and
selection; its report has no warnings or full-suite fallback, and timed calls
report `prepare`, `generic-checks`, then `total`.

## Invocation Lock And Timeouts

Every async analysis function except `version()` accepts these common options:

```ts
interface InvocationOptions {
  timeout?: number | null;
  lockTimeout?: number | null;
  failOnLock?: boolean;
  jobs?: number | null;
}
```

Durations are non-negative integer seconds. `timeout` limits command execution
after the lock is acquired, while `lockTimeout` limits only the lock wait. The
Node/N-API defaults disable both deadlines (`timeout`/`lockTimeout` omitted,
`0`, or `null`). The CLI still defaults to 30 seconds; pass `0` there to
disable. `failOnLock: true` fails immediately on contention and overrides
`lockTimeout`. `jobs` sets the rayon worker count for that invocation.
Pass `0` or omit it to use the CPU count (matching CLI `--jobs 0`). A
positive integer pins the pool on the first N-API call in the process.

`import-dynamic` follows every string-literal `import()`, including `if` /
`switch` / nested-function / JSX-handler loaders (`conditional-dynamic-import`
when the static call graph does not prove they run).

Literal dynamic imports resolve relative paths, workspace wildcard `exports`,
and package `#imports` through the prepared visible resolver catalog, including
inside test callbacks and `Promise.all`. Both `import` and `import-dynamic`
follow these targets without requiring `workspace`; adding `workspace` also
records `workspace` edges. Reverse `dependents` and `related` queries
use the same edges. Computed specifiers remain unresolved and are not guessed.

Forward `dependencies` reports stay on the lazy reachable-file walk when every
requested relationship is an import kind (`import`, `import-static`,
`import-dynamic`, `import-type`, `import-require`). `dependents` and `related`
reports still prepare reverse-index facts for the visible universe. Adding
`workspace`, `test`, `route`, or any other relationship also prepares facts for
the entire visible universe. Prefer import-only relationships plus explicit
`files` when the caller only needs a module closure. `analyzeProject()` without
a `check` report no longer eagerly parses every indexable file for an
import-only `dependencies` plan.

`candidateInclude` / `candidateExclude` belong only to forward import-only
`dependencies()` / `{ type: "dependencies" }` options (`TraverseGraphOptions` /
`TraversePathsOptions`). `dependents()`, `related()`, and
`{ type: "dependents" | "related" }` take `TraverseOptions` and do not accept
those fields or `projection`. The globs define the initial candidate
inventory before GraphFiles / fact preparation. They are not `filters` and
are not merged from top-level `analyzeProject` `filters`. Unrelated excluded
files are omitted from that inventory and receive no parse/fact work on a
standalone `dependencies()` call or an exclusive same-bounds import-only
`analyzeProject` request. Mixed requests that also run `check`, `dependents`,
or unbounded graph reports keep the shared GraphFiles universe, so those
other reports may still parse the excluded files. A
resolved local or workspace source that is in the pre-filter GraphFiles
universe still escapes into the closure. `projection: "paths"` returns
`{ files: string[], diagnostics: TsConfigDiagnostic[] }` — sorted unique
repository-relative paths plus bounded diagnostics — and does not change
which paths are walked. Omit `projection` or pass `"graph"` to keep
`DependencyResult`. These candidate options are also invalid at runtime on
`includeSymbols` or non-import relationships.
`analyzeProject` report `result` stays loosely typed: a `dependencies`
report body is `ImportClosureResult` when that report sets
`projection: "paths"`, and `DependencyResult` otherwise.

`resolveCheckDependencies` derives a batch `resolveCheck` result from named
`dependencies` reports in the same request. The derived report and referenced
reports are matched by effective `root`, `tsconfig`, and `config`; report-level
scope overrides are supported, and IDs are resolved only within that scope. It
checks the union of each report's seed files and reachable local files using
the already-prepared facts, resolver catalog, and source store. `targetModules`
and folder `filters` on a
referenced report are output projections; the derived check uses the
pre-projection file closure. Referenced reports may use import and
workspace relationships, and each needs an `id`; this report does not accept
`file` or `files`.

```js
const closure = await dependencies({
  root,
  files: ["web/app/page.tsx"],
  relationships: ["import-static", "import-dynamic", "import-type"],
  candidateInclude: ["web/**"],
  candidateExclude: ["**/*.test.*"],
  projection: "paths",
});
const report = await analyzeProject({
  root,
  reports: [{
    type: "dependencies",
    files: ["web/app/page.tsx"],
    relationships: ["import-static", "import-dynamic", "import-type"],
    candidateInclude: ["web/**"],
    candidateExclude: ["**/*.test.*"],
    projection: "paths",
  }],
});
```

```js
const report = await analyzeProject({
  root,
  reports: [
    {
      type: "dependencies",
      id: "route-closure",
      files: ["web/app/page.tsx"],
      relationships: ["import-static", "import-dynamic", "import-type"],
      projection: "paths",
    },
    {
      type: "resolveCheckDependencies",
      dependencyReportIds: ["route-closure"],
    },
  ],
});
```

The lock is shared by CLI and Node/N-API analyses for the current OS user across
all repositories. While waiting, stderr reports `waiting for lock held by pid
<pid> for <n>s`. Successful return values keep their existing shapes, and lock
or timeout failures reject the returned Promise with an actionable error. For
`analyzeProject()`, put these options at the top level,
not inside individual report requests:

```js
const report = await analyzeProject({
  timeout: 60,
  lockTimeout: 10,
  failOnLock: false,
  reports: [{ type: "dependencies", files: ["src/api.mts"] }],
});
```

Public JavaScript APIs take options objects and return parsed values. Native
JSON entrypoints accept and return Node `Buffer` values of UTF-8 JSON so the
addon avoids UTF-16 string copies at the N-API boundary.

## Agent Defaults

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
predicate contexts and EXISTS correlation. See [SELECT scope facts](postgres-source-api.md#select-scope-facts).
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
See [tracked route configuration](rules/nextjs-redirect-destinations.md#options).

### Selected TypeScript and JavaScript module facts

`await analyzeTypeScriptModules({ root, files: ["src/module.ts"] })` returns
`TypeScriptModulesReport`, with one `TypeScriptModuleFacts` per distinct selected
file, sorted by normalized absolute `fileName`. An explicit `root` must name an
existing directory. It reads and parses selected modules
once through the request-local source store and fact collection; it does not discover
other modules, resolve packages, or construct a dependency graph. CJS and ESM expose
the same async function, with the standard invocation controls.

Each module includes imports (default, named, namespace, side-effect, and type-only),
exports (named, local, default, star, and namespace re-exports), literal dynamic import
and global `require` loads, lexical scopes, bindings, and bound references. Import
specifier `bindingId` identifies its entry in `bindings`. `runtime` on a binding
means the declaration is syntactically a value; on a reference it means evaluation
uses that value. `typeOnly` distinguishes type uses, including `typeof Value` in a
type query. A value import used only in annotations has no runtime references.
Local named exports of type declarations are type-only even without an explicit
`type` modifier; source re-exports retain their syntactic flags.
An empty sourced export (`export {} from "./dep"`) retains its module request
with empty `local` and `exported` names.
`shadows` identifies the nearest enclosing same-name binding. IDs are local to a
module and are not persistent identities.

All spans are half-open **UTF-8 byte offsets** into the original source. They are not
JavaScript UTF-16 string indexes; use `Buffer.from(source).subarray(start, end)`.
String literals and templates without substitutions use the existing static import
extractor, including parenthesized TypeScript wrappers. Transparent wrappers on
`require` callees and default-export identifiers preserve their binding identity.
A default-export expression has an empty `local` name when its identifier does
not resolve to a binding in the selected file. No AST or raw source is
returned. Ambient declarations are not runtime values.
The facts do not prove that an imported value exists in another module.

Check `complete` and `diagnostics` before relying on a module. Non-literal module
loads, shadowed or indirect `require`, direct `eval`, `with`, TypeScript import-equals and legacy
export-assignment/namespace-export forms, runtime CommonJS export-object references
(including aliases, assignments, updates, and property-definition calls), and merged
declarations are explicit diagnostic gaps. Ambient modules and namespaces also
produce a gap; their nested import and export declarations are not file-level module facts. Source I/O,
unsupported source extensions, and parser/semantic errors also make the module
incomplete; any recovered facts remain available for inspection. Empty `files`
returns an empty report, without a global fallback.

<!-- cspell:ignore subarray -->



`parsePostgresSql()` preserves safely attributed constraint occurrences inside
nested conditional `DO` bodies, including original global spans, typed foreign
key references and `NOT VALID`. Incomplete procedural coverage retains localized
diagnostics alongside recovered facts. See [nested constraint source facts](postgres-source-api.md#constraints-inside-conditional-do-bodies).

`parsePostgresSql()` also returns `block.occurrences` for PL/pgSQL `DO` bodies.
Kinds are `utility`, `controlFlow`, `dml`, `dynamicExecute`, and `unknown`.
`CREATE TYPE` is `utility`. `IF`/`RAISE` with no DML is `controlFlow`. A loop
containing `INSERT`, `UPDATE`, `DELETE`, or `MERGE` nests a `dml` occurrence
and does not claim that statement executes. Dynamic `EXECUTE` stays
`dynamicExecute` and keeps the block incomplete. Nested literal `EXECUTE`
commands share the walker's 64-level budget and fail closed when it is exhausted.
A wholly literal `FOR ... IN EXECUTE` operand is `dml` or `utility`, including a
parsed `SELECT`. A qualified name such as `public.execute(...)` stays
`dynamicExecute` rather than static DML parsed from the argument. A literal
command followed by unquoted `INTO` is classified from that command, and the
`INTO` target still keeps the block incomplete. An IF, ELSIF, or
conditional-loop header with no condition keeps the block incomplete. That empty
header reports that the procedural condition is missing, at the header. An
opening label is one identifier; PL/pgSQL reserved words such as unquoted
`BEGIN` are not labels, while unquoted `SELECT` is allowed. Labels compare
after unquoted case folding and PostgreSQL's 63-byte UTF-8 identifier
truncation; quoted and unquoted forms can match when their stored names agree.
A closing label that does not match keeps the block incomplete and reports
`unknown` at that label, in source order. A label prefix on a wholly literal
command stays `unknown`. An opening label is omitted with the walker-only
block it names. See
[PostgreSQL source facts](postgres-source-api.md).

`parsePostgresSql()` and its batch overload expose data-modifying CTE bodies as
`query.nestedStatements`, with named exported statement, DML assignment, MERGE
and RETURNING types. Child IDs reference the same query report; source order,
completeness and exact SQL slices are described in [PostgreSQL source facts](postgres-source-api.md#data-modifying-ctes).

`parsePostgresSql()` exposes typed EXPLAIN/PREPARE execution wrappers and a
non-executing declaration wrapper on CREATE FUNCTION facts, including supported
SQL BEGIN ATOMIC child occurrences. See [statement wrappers](postgres-source-api.md#statement-wrappers-and-execution-context)
for completeness, diagnostics, and ancestor execution semantics.

Literal PL/pgSQL `EXECUTE` source occurrences in supported `DO` bodies expose
`kind: "literalExecute"` and a `PostgresSqlLiteralExecute` payload. Dollar-quoted,
standard single-quoted (doubled quotes), and PostgreSQL `E` escape strings are
decoded by the prepared tokenizer and parsed through the same SQL fact pipeline.
The enclosing statement and `literalSpan` retain original source coordinates;
`decodedSql` owns nested command statement, expression, and diagnostic coordinates.
Children preserve order and typed facts (including INSERT), without exposing an
AST or implying execution. Inspect `complete` and diagnostics: malformed nested
SQL remains diagnostic. Wholly literal `||` concatenations use the same nested
pipeline and expose `bodyEncoding: "concatenated"`; `literalSpan` covers the full
command expression. `using` retains ordered parameter expressions and their
original source spans, without resolving runtime values; `$1` and other command
placeholders remain typed parameters. Dynamic operands, `format` calls, and
unsupported EXECUTE modifiers remain incomplete `other` occurrences. An unquoted `INTO` target ends the command expression the same way `USING` does: the occurrence is classified from the literal, and the block stays incomplete. This adds no SQL execution or
replay policy. `parsePostgresSql` retains its asynchronous single/batch API.

`parsePostgresSql()` exposes `PostgresSqlConstraint.span` for CREATE TABLE inline
and table constraints, ALTER TABLE ADD COLUMN inline constraints, and ALTER
TABLE ADD CONSTRAINT. The nullable span uses
exact source byte offsets, including comments and validation suffixes, without
changing statement locations or formatted `constraint.sql`. See
[constraint source spans](postgres-source-api.md#constraint-source-spans) for
coordinate ownership and absence semantics.

The opt-in [`query-reached-per-item`](rules/query-reached-per-item.md) rule runs
through asynchronous `check()` and `analyzeProject()` check queries. Configure
effect families, transaction sinks, batch exemptions, and rollout allowlists in
the same config used by the CLI; finding and suppression shapes are unchanged.

`parsePostgresSql()` exposes structured trigger `eventFacts`, cast
`dataTypeFacts`, and literal EXECUTE `using` expressions through the same async
single and batch APIs. The exported `PostgresSqlTriggerEvent`,
`PostgresSqlTriggerEventKind`, and `PostgresSqlExecuteEncoding` contracts preserve
display text while providing typed identifiers and command provenance. See
[structured trigger events and cast types](postgres-source-api.md#structured-trigger-events-and-cast-types).
