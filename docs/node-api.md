# Node/N-API Guide

The `no-mistakes` npm package exposes async functions backed by the same Rust
analysis as the CLI. Use it for repeated structured queries without subprocess overhead.

`check()` and check reports in `analyzeProject()` apply the shared
`unanalyzable` policy to `github-actions-checkout-credentials`: unknown
persistence or permissions, custom tokens, and malformed YAML report by
default, advise without failing, or are ignored. Both APIs share suppression
and source locations with the CLI. Reusable-workflow literal permissions are
known upper bounds; unsafe write bounds remain findings under all policies,
with job permissions taking precedence over workflow defaults. Only checkout
steps are judged, so a valid `pull_request_target` workflow without checkout
stays clean.
`pull_request_target` also checks PR-head refs independently of persistence;
unresolved refs/repositories use the shared policy. Configure the optional
`defaultBranch` convention in YAML selected by `configPath`, not as a `check()`
argument. The Node/N-API signature and report types are unchanged. A checkout
can have separate credential and ref rows. See the
[rule contract](rules/github-actions-checkout-credentials.md#analysis).

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

For `shellcheck-runner`, both APIs apply the same repository or
rule-application `unanalyzable` policy when the ShellCheck executable is
missing or a failed invocation yields no recognized diagnostics. If an
abnormal exit has some recognized diagnostics, those findings are preserved
and the policy applies to selected scripts without recognized diagnostics.
Other launcher errors, such as permission denied, are hard errors. The default
is `report`; `advise` returns a non-failing advisory and `ignore` skips that
uncertainty. See [the rule contract](rules/shellcheck-runner.md#what-it-catchesrequires).

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
For `playwright-unique-test-ids` and `playwright-coverage`, `low` accepts a
string literal test id, including JSX `data-testid={"save"}`, or a prop with a
literal default; other selector shapes use the same unanalyzable policy. See
[playwright-unique-test-ids](rules/playwright-unique-test-ids.md#effort) and
[playwright-coverage](rules/playwright-coverage.md#effort).
For `playwright-unique-html-ids`, `low` checks literal HTML IDs and routes
other ID expressions through the shared unanalyzable policy. The rule remains
independent of `tests.playwright.selectors.htmlIds`; see
[its Effort section](rules/playwright-unique-html-ids.md#effort).
For `nextjs-no-caching`, `low` checks
[literal cache settings](rules/nextjs-no-caching.md#effort) without preparing
constant propagation or configuration wrapper facts. Both APIs preserve shared
unanalyzable policy and suppression behavior; mixed low/max applications reuse
the same source inventory and parse.
For `require-storybook-stories`, `low` keeps
[direct story coverage](rules/require-storybook-stories.md#effort) and configured
colocated-test coverage without transitive render-chain or dynamic/mock boundary
analysis. `check({ effort: "low" })` and a check report in `analyzeProject()` use
the same shared unanalyzable policy as CLI `--effort low`.
For `postgres-no-offset`, `low` checks
[literal SQL](rules/postgres-no-offset.md#effort) without helper evaluation.
`check({ effort: "low" })` and check reports in `analyzeProject()` use the same
unanalyzable policy, source locations, suppression, and SQL verdict controls as
the CLI. Raw SQL files retain their existing analysis.

For `finite-set-consistency`, `low` evaluates YAML sequences, YAML string
selectors, and path-regex captures. Other extractor kinds use the same
unanalyzable policy as the CLI, and comparisons that involve those sets are
skipped even when their residual is ignored or suppressed. `minSize` does not
run on unsupported sets. Supported-set mismatches remain definite findings;
use `effort: "max"` to analyze TypeScript, SQL-enum, and Markdown sets.

For `unconstructed-error-class`, unresolved constructor targets and otherwise-unused
classes with unresolved constructors or statically named unresolved bases produce
non-failing advisories even with `options.unanalyzable: report` (the default).
`advise` also advises; `ignore` omits these diagnostics. No `unanalyzable`
value turns an unresolved constructor or a statically named unresolved base
into a failure. Dynamic base ancestry follows the shared policy.
`report` and `advise` differ for dynamic base ancestry and when namespace
escape is the class's only uncertainty.
Namespace-escape-only uncertainty keeps the shared policy: `report` fails,
`advise` advises, and `ignore` omits it. Definite unused Error classes remain
failures under every policy. Known constructor targets reuse the prepared
resolver, including path aliases, usable package declarations, and
tsconfig-visible ambient modules, and credit the actual class.
Immutable aliases and package constructor facades can resolve through shared
binding facts and concrete `typeof` class declarations. Unknown alternatives
remain uncertain; declaration proof does not broaden ordinary query scope.
Annotated constants follow their runtime initializer through barrels, and
dependency lookup uses the re-exporter's package context. Conditional package
entries and separate `@types` declarations honor the static import/re-export
context: `.mts`/`.mjs` selects `import`, `.cts`/`.cjs` selects `require`, and
ambiguous extensions retain agreement checks across branches. This also covers
default-object destructuring. CLI and aggregate API reports apply
the same proof: non-Error targets are silent, while unused Error subclasses
remain findings under every policy.
Known non-Error globals are quiet; shadowed names resolve independently.
Ordinary import/call query results remain unchanged. Suppressions apply at the
construction or class line in both report channels. The rule remains `max`
only; a requested `low` warns and runs `max`.

For `version-pin-consistency`, CLI `check`, `check(options)`, and check reports
in `analyzeProject()` apply the same `unanalyzable` policy when a selected
anchor's declared comparison source is outside the selected file set and is
missing, unreadable, or malformed, or when a selected anchor itself cannot be
read as UTF-8. Actual read failures on the selected anchor use this policy too.
`report` fails by default, `advise` emits non-failing advice, and `ignore`
suppresses the uncertainty. A valid source outside the selected file set still
supplies the comparison pin, and definite mismatches remain failures.

For `github-actions-script-expressions`, CLI `check`, `check(options)`, and
check reports in `analyzeProject()` apply the same `unanalyzable` policy when a
selected workflow or composite-action file cannot be read, including invalid
UTF-8. `report` fails by default, `advise` emits non-failing advice, and
`ignore` omits the uncertainty. Invalid YAML, unmatched source locations, and
unsupported expression shapes retain definite findings under every policy.
Both findings and advice honor suppression. Invalid UTF-8 points to its first
invalid byte's line; other read failures use line 1.

For `package-json-required-fields`, CLI `check`, `check(options)`, and check
reports in `analyzeProject()` apply the same `unanalyzable` policy to selected
manifests that cannot be read or parsed when a field check is enabled.
`report` fails by default, `advise` returns a non-failing advisory, and `ignore`
omits the uncertainty. Definite field violations remain failures, and empty
options request no checks. Both diagnostic channels honor suppression. See
[the rule contract](rules/package-json-required-fields.md#unanalyzable-manifests).

For `github-actions-job-timeouts`, `check({ effort: "low" })` and a check report
in `analyzeProject()` use the same covering `github-actions-step-timeouts`
partner as CLI `--effort low`. With a certified step cap, the job rule skips
redundant `rejectStepExceedingJob` comparisons only for literal job timeouts at
least that cap; the partner reports step violations. Missing or ineligible
coverage produces the same warning, `analysisFallbacks` record, and `max`
fallback in both APIs. See [its Effort section](rules/github-actions-job-timeouts.md#effort).
For `github-actions-download-integrity`, `low` skips assignment expansion for
URL values and routes a known downloader command with any dynamic candidate URL
through one shared unanalyzable diagnostic. Literal-only commands retain their
checks in CLI and Node API. The rule may inspect one binding to identify a
dynamic command prefix. See
[its Effort section](rules/github-actions-download-integrity.md#effort).
For `declared-payload-compatibility`, `low` checks scalar roots, flat objects
with scalar properties, and root arrays with scalar items. Nested child objects
or arrays use the same contract-level unanalyzable policy in `check()` and
`analyzeProject()` check reports; invalid documents, pointers, or configuration
remain failures. See
[its Effort section](rules/declared-payload-compatibility.md#effort).

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
| `check`                                    | `check(options)` (including configured `forbidden-lint-suppressions`)                                                                                                                                                                                                                                                           |
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
| `check` | `check(options)`; configured `forbidden-lint-suppressions` uses `checkJson` |
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

`check()` and `analyzeProject()` also honor low effort for [`postgres-no-unbounded-sql-identifier`](rules/postgres-no-unbounded-sql-identifier.md#effort). Import the exported `PostgresNoUnboundedSqlIdentifierOptions` type for its configuration. Unsupported syntax uses the same report/advisory/ignore policy in CLI and async Node reports.

The `postgres-no-mutable-sql-registry` check accepts the exported
`PostgresNoMutableSqlRegistryOptions` configuration type. Its
[`effort: "low"` tier](rules/postgres-no-mutable-sql-registry.md#effort)
checks direct literal SQL through named executor imports without preparing
cross-file mutation analysis;
unsupported inputs use the shared `unanalyzable` report/advice/ignore policy.

Configured [`postgres-no-test-state-mutation`](rules/postgres-no-test-state-mutation.md)
applications run through the existing async `check()` binding and through
`analyzeProject()` check reports, with the same findings and advisories as CLI
`check`. Import the exported `PostgresNoTestStateMutationOptions` type for
`testGlobs`, `helperRoots`, executor targets, and the unanalyzable policy. The
runtime export remains `check(options)`; this rule adds no separate function.

The async `check()` API accepts `PostgresRequireQueryAnnotationOptions.sqlLiteralHelpers` in rule configuration, for example `["pg-copy-streams#from"]`. Both effort modes recognize direct imported helper calls with one static literal argument; `SqlExecutorProfile.sqlLiteralHelpers` records the configured identities in requested SQL verdicts. See [the annotation rule](rules/postgres-require-query-annotation.md#configured-single-literal-sql-helpers).

`check()` and check reports in `analyzeProject()` support
[`postgres-no-transformed-uuid-predicates`](rules/postgres-no-transformed-uuid-predicates.md).
Import `PostgresNoTransformedUuidPredicatesOptions` to name its catalog, executor,
and effort options. Both APIs preserve findings and non-failing advisories.

Playwright file selection uses the same static string/RegExp/mixed-array config
matchers as the CLI. `check()` and `check` reports in `analyzeProject()` retain scoped
matcher uncertainty in their existing `warnings` field; see
[Playwright file matchers](configuration/tests.md#playwright-file-matchers).
`playwrightCheck()` includes an optional `warnings` array for the same uncertainty;
the field is omitted when all configured matchers can be analyzed. Import
`PlaywrightCoverageReport` to name the warning-bearing result.

`check()` also runs configured [coverage-producer-assertion-owner](rules/coverage-producer-assertion-owner.md) applications, returning ordinary `rules` findings or `advisories`. Its exported option types are `CoverageProducerAssertionOwnerOptions`, `CoverageProducerFunction`, `CoverageAssertionImport`, and `CoverageAssertionOwnership`. The existing asynchronous `check` → `checkJson` binding remains the runtime entrypoint.

`check()` and batched `analyzeProject()` check reports also run configured
[forbidden-lint-suppressions](rules/forbidden-lint-suppressions.md) applications.
Import `ForbiddenLintSuppressionsOptions` and
`ForbiddenLintSuppressionDirective` for their public configuration contract.
The existing asynchronous `check` → `checkJson` binding returns ordinary
`RuleFinding` rows: `line` identifies the directive and `target` identifies the
forbidden lint ID, `*` for a selected blanket disable, or the explicitly enabled
TypeScript directive name. No additional runtime export is required.
