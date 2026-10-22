# Node/N-API Guide

The `no-mistakes` npm package exposes async functions backed by the same Rust
analysis as the CLI. Use it for repeated structured queries without subprocess overhead.

```js
const { analyzeProject } = require("no-mistakes");
(async () => {
  console.log(await analyzeProject({ root: process.cwd(), reports: [{ type: "check" }] }));
})();
```

- [Getting started](node-api/getting-started.md)
- [PostgreSQL executor extraction](node-api/postgres-extraction.md)
- [Check SQL verdicts](node-api/check-verdicts.md): per-obligation proof ledger and residual advice grouped per executor call/application
- [PostgreSQL catalog generation](node-api/postgres-catalog.md)
- [Planning and graph APIs](node-api/planning-and-graphs.md)
- [Test selection and analysis scope](node-api/test-selection.md)
- [Resolution and workflow APIs](node-api/resolution-and-workflows.md)
- [Invocation Lock And Timeouts](node-api/invocation.md)
- [Agent Defaults](node-api/agent-defaults.md)
- [Selected TypeScript and JavaScript module facts](node-api/typescript-modules.md)
- [Declared runner config deadlines](node-api/runner-deadlines.md)
- [Live phase progress](node-api/progress.md)
- [Effect alternatives](node-api/effect-alternatives.md): retained possible receiver targets and physical call matches

## CLI Mapping

`check(options)` and check reports in `analyzeProject()` accept the same
`effort: "low" | "max"` option as CLI `--effort`. See
[check options](node-api/check-verdicts.md) for precedence and fallback
warnings.

For `postgres-require-query-annotation`, `low` uses the same
[literal-head proofs](rules/postgres-require-query-annotation.md#effort) and
unanalyzable policy in both APIs; it omits annotation evaluator preparation.
For `nextjs-redirect-destinations`, `low` keeps
[literal arrays and objects](rules/nextjs-redirect-destinations.md#effort) and
routes every other destination construction through the same unanalyzable
policy.
For `test-no-unmocked-dynamic-imports`, `low` accepts a top-level static
import, a `vi.mock` or `jest.mock` call, or a literal `require()` in setup
files; other setup-reachability shapes use the same unanalyzable policy. See
[its Effort section](rules/test-no-unmocked-dynamic-imports.md#effort).

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
| Source-only TS/JS executor SQL extraction | `extractExecutedPostgresSql({ sources, importSpecifier, includeDiagnostics: true })` |
| Selected TS/JS modules | `analyzeTypeScriptModules({ root, files })` |
| `flow`                                     | `flow(options)`                                                                                                                                                                                                                                                            |
| Global `--progress` | Pass `diagnosticsId` and poll `diagnosticsSnapshot({ id })`; stderr-free live snapshots |
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
| `tests audit` | `testsAudit(options)` compares versioned plan/observation artifacts; see [artifact contract and producer recipe](cli/tests-audit.md) |
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

## Runtime exports

| Runtime export | API |
| --- | --- |
| `createWorkflowTopologyIndex` | `createWorkflowTopologyIndex(topology)` |
| `version` | `version()` |
| `diagnosticsSnapshot` | `diagnosticsSnapshot({ id })` |
| `analyzeProject` | `analyzeProject(options)`; import `AnalyzeProjectReportType` for report `type` instead of indexing `AnalyzeProjectReportRequest['type']` |
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
| `extractExecutedPostgresSql` | `extractExecutedPostgresSql({ sources, importSpecifier })` |
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
| `testsAudit` | `testsAudit(options)` |
| `testsGraph` | `testsGraph(options)` |
| `testsImpact` | `testsImpact(options)` |
| `testsPlan` | `testsPlan(options)` |
| `testsTargets` | `testsTargets(options)` |
| `testsWhy` | `testsWhy(options)` |

## PostgreSQL catalog generation

See [PostgreSQL catalog generation](node-api/postgres-catalog.md).

## Invocation Lock And Timeouts

See [Invocation Lock And Timeouts](node-api/invocation.md).

## Agent Defaults

See [Agent Defaults](node-api/agent-defaults.md).

### Selected TypeScript and JavaScript module facts

See [Selected TypeScript and JavaScript module facts](node-api/typescript-modules.md).

### Declared runner config deadlines

See [Declared runner config deadlines](node-api/runner-deadlines.md).

## Live phase progress

See [Live phase progress](node-api/progress.md).

### Effect alternatives

See [Effect alternatives](node-api/effect-alternatives.md).
