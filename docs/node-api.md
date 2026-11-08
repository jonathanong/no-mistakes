# Node/N-API Guide

The `no-mistakes` npm package exposes async functions backed by the same Rust
analysis as the CLI. Use it for repeated structured queries without subprocess overhead.

Configured `workflow-topology-policy.options.forbiddenActivityTypes` runs through
async `check({ root, config })` and `analyzeProject()` check reports with the same
exact matching, event filtering, `types` key locations, custom messages, and
suppressions as CLI `check`. Configure it in the YAML selected by `config`;
there is no additional runtime export or invocation option. The existing
`check` → `checkJson` binding carries this rule. See
[activity type configuration](configuration/workflow-topology-policy-activity-types.md).

`check({ root, config })` and `{ type: "check", config }` reports in
`analyzeProject()` honor the same `forbidden-dependencies` glob roots, root
exclusions and per-root file exceptions as the CLI. Configure them in the YAML
file selected by `config`; the exported `ForbiddenDependenciesOptions` and
`ForbiddenDependencyException` types describe these rule options. Dependency
resolution and transitive paths use the shared canonical graph. See the
[rule's glob and exception contract](rules/forbidden-dependencies.md#glob-semantics).

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

The configured [`github-actions-job-permissions`](rules/github-actions-job-permissions.md)
rule also runs through the existing asynchronous `check()` and
`analyzeProject()` check-report APIs, with the same findings, advisories, and
suppression behavior as CLI `check`. Its unresolved permission declarations
use the shared `unanalyzable` policy; definite inherited write grants remain
findings under every policy. The rule adds no function, option, or report
type, so `check()` remains the runtime entrypoint.

`query-reached-per-item` accepts `exemptCappedCursorPageReads` in the rule's
YAML `options`, selected by `check({ root, config })` or an `analyzeProject()`
check report. It defaults to `false` and shares CLI behavior. The exported
`QueryReachedPerItemOptions` and `QueryReachedPerItemAllowEntry` describe that
configuration; the async runtime entrypoints and report fields are unchanged.
Allow entries accept an optional `function` string to match the exact enclosing
caller name, combined with any `callee` and `line` selectors. File-only entries
retain their behavior, including unused-entry reporting.
See the [cursor exemption contract](rules/query-reached-per-item.md#capped-sequential-page-reads).

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

`check()` and `{ type: "check" }` reports from `analyzeProject()` include an
optional `CheckReport.incomplete` array when a requested consumer cannot finish
because a prerequisite failed. Each exported `CheckFailure` identifies the
consumer (a rule identifier, configured trait, or check domain), optional source file and
configuration property, and diagnostic message. Successful consumers still
return their findings. These failures are neither findings nor advisories and
cannot be suppressed. The field is omitted when no prerequisite failures were
captured; its absence does not mean arbitrary dynamic JavaScript configuration
was evaluated. These isolated source or configuration failures resolve to the
partial report rather than rejecting the API call. Global invocation or
configuration errors that make the request unusable still reject the API call.
The CLI reports incomplete consumers and exits 1 for the same isolated
failures. Prerequisites follow each consumer's source and configuration input
selection, independently of other enabled rules. Binary assets, directories, and
dangling links outside that selection do not make a consumer incomplete. A required
source that fails to read or parse still names that consumer and input.
Per-file foreign receiver work-budget exhaustion uses the same `incomplete`
contract, with the affected file and an actionable message. `check()` and
`analyzeProject()` discard partial value-flow facts and preserve independent
source-check findings. Mixing a check with import-only dependency or dependent
reports preserves each standalone result, including partial check findings.
Import-only `forbidden-dependencies` checks retain their findings; repeated
partial checks share the complete canonical import projection.
Narrow `root` or simplify shared receiver mutations;
application `include`/`exclude` filters do not narrow the full graph evidence.
Reverse-query diagnostics classify exhausted value flow as `operational-error`.
Vitest config failures identify the config file and property (for example,
`test.include`), with the source line in the message.

A rule application with `severity: advisory` (see
[rule applications](configuration/rules.md#severity)) moves that rule's
findings from `CheckReport.rules` and `CheckReport.codebase` to
`CheckReport.advisories` in `check()` and `analyzeProject()`. The finding shape
is unchanged; only the list, and therefore the failure decision, differs.

`check()` and `analyzeProject()` reports with `type: "check"` audit unused
no-mistakes disable comments using the selected configuration's
`reportUnusedDisableDirectives` setting. The default `advise` puts
`unused-disable-directive` and `invalid-disable-directive` diagnostics in
`CheckReport.advisories`; `report` puts them in `CheckReport.rules`, and
`ignore` omits them. Locations refer to the comment. Unknown and disabled rule
IDs have distinct messages. `includeSuppressed` controls only the optional
ledger of hidden findings. See [suppression hygiene](configuration/suppressions.md).

Suppression hygiene in `check()` and `analyzeProject()` check reports includes
stale next-line directives inside statically recovered PostgreSQL executor SQL.
It follows `reportUnusedDisableDirectives` and uses the same physical comment
locations as CLI `check`; existing report types and async entrypoints apply.

Both APIs also honor repository `reportUnusedAllowEntries` for proven stale
configured exceptions: `advise` (default) returns `unused-allow-entry` in
`CheckReport.advisories`, `report` returns the same diagnostic in
`CheckReport.rules`, and `ignore` omits it. The setting never expands analysis
scope; incomplete selector coverage stays silent. Literal file paths within the
configured scope also prove staleness when absent from both inventory and disk.
Existing excluded or ignored files and empty glob matches stay silent. The exported
`UnusedAllowEntryPolicy` union names this policy in `DeclaredConfig`.

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
CLI and async Node checks also share the low-effort proof for a local primitive
`let` string with a continuation-safe head, discarded literal/template `+=`
writes, and only direct configured executor reads in the same function.
Conditional appends and template interpolations need no SQL evaluation;
captures, aliases, reassignment, and dynamic ownership gaps remain unresolved.

These proofs include trusted builder append prefixes and fixed-return helpers
in the same file or through one direct source import, including local export
aliases and concise arrow bodies. Opaque append suffixes preserve the prefix
only for the trusted `sql-template-strings` tag. Unsupported helper or
mutation shapes remain unanalyzable.
The same prepared proof joins two adjacent exhaustive assignments to one local
`let` when each arm creates a fresh native tagged builder and the only later
use is its immediate configured executor read. It retains missing-head findings
and unresolved mixed or escaped forms without running the annotation evaluator.
They also include an exact directly imported helper that returns an immutable
append alias of one fresh tagged builder argument, with an inert native default
and a closed paired `.text`/`.values` executor read.
The one-hop proof also accepts a one-parameter imported helper whose complete
body creates a const append alias with literal-only suffixes and returns it.
Both conditional helper-call arms must prove the head; the private local result
may receive discarded literal/tagged appends before its paired field read.
Statements received as function parameters remain unanalyzable; see the
[low-effort contract](postgres-query-annotation/low-effort.md).
An exact top-level namespace destructure can follow unique explicit source
reexports to a fixed-head helper through the same prepared check projection;
other namespace shapes remain unresolved. At low effort, a helper's own
returned trusted builder may keep its fixed head through captured append-only
closures when every binding use is proven safe.
Captured trusted local builders also qualify when every exact-binding use is a
discarded append, a configured executor read, or a proven local append-only
effect. This preserves only the fixed head, even across arbitrary closure order.
The shared low proof also accepts bounded same-file append-only helper calls
and a returned parameter-rooted append chain when the caller discards it. An
append-only helper's captured callback may make discarded static appends to
its exact parameter, including after or between executor reads.
An unsupported use anywhere in a local builder's lifetime invalidates every
read of that builder, even reads before a mutation or escape. Plain immutable
strings keep their existing head proof.
The same prepared effect proof permits inert literal defaults on later helper
parameters and closed object arguments for supported destructured defaults.
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
[literal SQL and the bounded helper/builder subset](rules/postgres-no-offset.md#effort).
`check({ effort: "low" })` and check reports in `analyzeProject()` use the same
unanalyzable policy, source locations, suppression, and SQL verdict controls as
the CLI. Configured SQL-template interpolations are bind values; unsupported
shapes follow the shared policy, and raw SQL files retain their existing
analysis. See the [bounded helper contract](postgres-facts/low-helper-recovery.md).
The `postgres-required-predicates` bounded-index-probe exemption also applies to
programmatic checks: it requires configured catalog index facts, a leading-column
equality, and the SELECT's own constant or parameter LIMIT. See the
[rule's examples and controls](rules/postgres-required-predicates.md#bounded-index-probes).

`postgres-explicit-columns` and `postgres-required-predicates` also support
`low` through `check()` and `analyzeProject()` with the same bounded SQL helper
subset and `unanalyzable` policy as the CLI. See the
[explicit-columns](rules/postgres-explicit-columns.md#effort) and
[required-predicates](rules/postgres-required-predicates.md#effort) Effort
sections.
[`postgres-no-test-state-mutation`](rules/postgres-no-test-state-mutation.md#effort)
also supports `low` in `check()` and `analyzeProject()` for configured test SQL;
bounded source helpers and finite inline Vitest rows use the same prepared
proofs as the CLI. Opaque tails retain the shared unanalyzable policy.

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
| Global `--verbose-timings` | `collectDiagnostics(operation, ...args)` returns `{ result, diagnostics }` without stderr |
| Global `--progress` | Pass `diagnosticsId` and poll `diagnosticsSnapshot({ id })`; stderr-free live snapshots |
| `check`                                    | `check(options)` (including configured `forbidden-lint-suppressions` and `vitest-project-policy`)                                                                                                                                                                                                                                                           |
| `config resolve [--view declared]`           | `resolveConfig(options)`                                                                                                                                                                                                                                                   |
| `lock status` | `lockStatus()` (read-only shared-lock probe; does not wait) |
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
| `collectDiagnostics` | `collectDiagnostics(operation, ...args)`; exported `DiagnosticOperations` preserves operation argument and result types |
| `diagnosticsSnapshot` | `diagnosticsSnapshot({ id })` |
| `analyzeProject` | `analyzeProject(options)`; import `AnalyzeProjectReportType` for report `type` instead of indexing `AnalyzeProjectReportRequest['type']` |
| `writePlanningImpactArtifacts` | `writePlanningImpactArtifacts(options)` |
| `callSites` | `callSites(options)` |
| `check` | `check(options)`; configured `forbidden-lint-suppressions` and `vitest-project-policy` use `checkJson`; reports may include `CheckReport.incomplete` (`CheckFailure[]`) |
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
| `resolveConfig` | `resolveConfig(options)` returns the existing resolved report; `resolveConfig({ view: "declared" })` returns `DeclaredConfig` through the same async `resolveConfigJson` binding. |
| `lockStatus` | `lockStatus()` |
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

`check()` and `analyzeProject()` also honor low effort for
[`postgres-no-unbounded-sql-identifier`](rules/postgres-no-unbounded-sql-identifier.md#effort).
Import the exported `PostgresNoUnboundedSqlIdentifierOptions`,
`PostgresSqlIdentifierValidator`, and `PostgresSqlIdentifierSource` types for
`identifierValidators` (`module?`, `name`, `argument`), `identifierSources`
(`module?`, `name`), and compatible `finiteDomainAssertions`. At max effort,
a completed configured validator call proves its argument at later uses and
its returned value. Trusted validators may enforce an allow-list or an
identifier grammar plus length limit; trusted sources promise a bounded scalar
identifier return. Their bodies are not inspected. Module selectors match the
call-site import or canonical resolved origin through named or `export *`
barrels and npm re-exports. Guards still cover only the exact validated property
path before interpolation.

Workspace packages and `export *` barrels also preserve configured source
identity for typed catalog producers populated at module load. A source entry
counts as used only when its scalar return supplies a selected SQL identifier
proof; an unrelated call or returned-object property does not use it. CLI and
async Node reports from complete max-effort scans locate unused selectors on
their config items as `unused-allow-entry`, honoring `reportUnusedAllowEntries`.

These async APIs share the CLI proofs for imported literal-array guards,
immutable templates and additions whose parts are literal or bounded at capture,
literal properties selected from closed records, closed catalog map/join SQL,
built-in `sql-template-strings` `.text` fragments with append chains and helper
returns, including imported statements and synchronous factories used through
configured executor callback types. Readonly import proofs reject producer or
consumer member writes and namespace escapes that expose the statement.
Generated UUID helper prefixes must receive literal call arguments, and async
forwarding must be awaited. No tag option is required for the default or `SQL`
import from `sql-template-strings`. Unsupported syntax uses
the same report/advisory/ignore policy in CLI and async Node reports. See
[closed domains](rules/postgres-no-unbounded-sql-identifier.md#closed-domains).

For `postgres-require-query-annotation`, these async reports share the CLI's
low-effort finite conditional heads, finite explicit helper returns, safe
returned-local builder proofs, and bounded literal names from direct tuple
loops, a two-literal name ternary, a string-literal `for-of`, and closed
local helper calls. An unresolved import leaves the ternary and string-list
forms under `unanalyzable`. Unsupported arms and escaped builders
retain the configured `unanalyzable`
policy. See [low-effort annotation](postgres-query-annotation/low-effort.md).

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
The shared prepared facts can also enumerate literal SQL fragments selected
by a local map from an exact imported Vitest `it.each` literal key array.
It also enumerates a local literal `Set` domain after a dominating membership
guard throws on nonmembers, with the same CLI and async API result.
The prepared helper projection also resolves fixed SQL switch arms whose
default is proven to throw, including a source-visible imported thrower.
It composes finite literal-string returns into complete ordinary SQL templates
before the shared policy check; tagged-template substitutions remain binds.
Local builder assignments after a direct throwing branch use the same
reachable SQL alternatives in CLI and async Node check reports.
Destructured and bound methods selected by `executorNames` on a configured imported receiver,
including local aliases, use the same CLI, CJS, ESM, and prepared check behavior.
Non-call runtime references retain the configured unanalyzable policy even if
a later direct call is analyzed. Defaulted destructuring parameters remain
uncertain; type-only references are excluded. This adds no public option or
separate API function.

The async `check()` API accepts `PostgresRequireQueryAnnotationOptions.sqlLiteralHelpers` in rule configuration, for example `["pg-copy-streams#from"]`. Both effort modes recognize direct imported helper calls with one static literal argument; `SqlExecutorProfile.sqlLiteralHelpers` records the configured identities in requested SQL verdicts. See [the annotation rule](rules/postgres-require-query-annotation.md#configured-single-literal-sql-helpers).

`check()` and check reports in `analyzeProject()` support
[`postgres-no-transformed-uuid-predicates`](rules/postgres-no-transformed-uuid-predicates.md).
Import `PostgresNoTransformedUuidPredicatesOptions` to name its catalog, executor,
and effort options. Both APIs preserve findings and non-failing advisories.

The async `check()` API and `analyzeProject()` check reports also support
[`postgres-json-relation-ids`](rules/postgres-json-relation-ids.md). Import
`PostgresJsonRelationIdsOptions` and `PostgresJsonRelationIdsAllowEntry` to name
its configuration. Its findings remain non-failing advisories in both APIs.

Playwright file selection uses the same static string/RegExp/mixed-array config
matchers as the CLI. `check()` and `check` reports in `analyzeProject()` retain scoped
matcher uncertainty in their existing `warnings` field; see
[Playwright file matchers](configuration/tests.md#playwright-file-matchers).
`playwrightCheck()` includes an optional `warnings` array for the same uncertainty;
the field is omitted when all configured matchers can be analyzed. Import
`PlaywrightCoverageReport` to name the warning-bearing result.

`check()` also runs configured [coverage-producer-assertion-owner](rules/coverage-producer-assertion-owner.md) applications, returning ordinary `rules` findings or `advisories`. Its exported option types are `CoverageProducerAssertionOwnerOptions`, `CoverageProducerFunction`, `CoverageAssertionImport`, and `CoverageAssertionOwnership`. The existing asynchronous `check` → `checkJson` binding remains the runtime entrypoint. Uncertainty messages in `rules` and `advisories` name the unresolved call site and callee or the traversal option, limit value, and last module. Required source failures stay in `incomplete`, with the file and original error in the message. The CLI and async Node API return the same cause details. Those findings also carry an optional `causes` array (`FindingCause`: `kind`, `file`, `line?`, `callee?`, `detail`) capped at 25 entries; the message lists at most five and ends with `+N more`. `RuleFinding.causes` is omitted for ordinary findings, and option types are unchanged.

`check()` and batched `analyzeProject()` check reports also run configured
[forbidden-lint-suppressions](rules/forbidden-lint-suppressions.md) applications.
Import `ForbiddenLintSuppressionsOptions` and
`ForbiddenLintSuppressionDirective` for their public configuration contract.
The existing asynchronous `check` → `checkJson` binding returns ordinary
`RuleFinding` rows: `line` identifies the directive and `target` identifies the
forbidden lint ID, `*` for a selected blanket disable, or the explicitly enabled
TypeScript directive name. No additional runtime export is required.

Configured [`production-exports-only-used-by-tests`](rules/production-exports-only-used-by-tests.md) applications run through the existing async `check()` → `checkJson` binding and `analyzeProject()` check reports. `ProductionExportsOnlyUsedByTestsOptions` exports its explicit production/test/helper scope, opt-in `ignoreExportsUsedInFile` boolean (default `false`), and shared uncertainty policy; results use ordinary check findings and advisories.

## `test-no-live-repository-scan`

[`test-no-live-repository-scan`](rules/test-no-live-repository-scan.md) runs
through the existing asynchronous `check()` and `analyzeProject()` APIs; it adds
no new entrypoint. Type its options with `TestNoLiveRepositoryScanOptions`
(including `runtimeAssets` entries, `TestNoLiveRepositoryScanRuntimeAsset`),
exported from `no-mistakes/types`. Runtime-asset reads are advisories with
target `runtime asset` in the check report.

The reports share CommonJS binding provenance with the canonical call graph,
retain unresolved computed loads, and distinguish shared temporary roots from
owned `mkdtemp` directories. Direct `import.meta.glob` and `fs-extra` scans follow
the same rule contract. CLI and compiled CJS, ESM, and prepared API reports are
locked by saved-fixture snapshots.
Node-origin `module.require` and imported `createRequire` loads use conservative
unresolved-root findings in these same async APIs, including computed module
arguments and simple loader aliases. They add no public module-fact fields.

## PostgreSQL query annotation

For PostgreSQL query annotation checks, low effort recognizes direct `.text`/`.values` reads from a safe local builder tagged by the exact `sql-template-strings` import. The asynchronous check API also projects one direct immutable string export through the prepared resolver.

The async query-annotation check shares low's bounded fresh helper-result proof
for paired `.text`/`.values` reads; it does not run the annotation evaluator.
The pair may be followed by one options argument when its builder result has a
closed source-visible origin.
The same low proof accepts a native builder's paired fields in a nested closure
only when every reference preserves its fixed head, even if that closure runs
later or more than once.
The same prepared proof covers an immutable local record text template and
one exact `{ text, values }` extraction from a fresh plain record.
It also accepts an exact fresh trusted builder returned into one local `const`
or directly appended at the executor, with the same one-hop prepared import
and escape guards as CLI checks.
The same prepared facts project an exact imported helper's fixed callback SQL
head to a direct inline configured executor callback, independently per call.
The async API shares low's same-file fixed-builder callback proof and exact
async callback parameter census without invoking the annotation evaluator.

The async `check()` and `analyzeProject()` annotation reports also share low's
[awaited fresh native return-origin proof](postgres-query-annotation/low-effort.md#awaited-fresh-native-return-origins).
It uses the same prepared source/effect projections and requires exact awaited
`const` consumption by the immediately next configured executor. No API option,
report field or max-effort contract changes.

Configured [`postgres-bounded-statements`](rules/postgres-bounded-statements.md)
applications accept `materializedBodies` and `columnBounds` through the existing
asynchronous `check()` →
`checkJson` binding and `analyzeProject()` check reports. Import
`PostgresBoundedStatementsOptions`, `PostgresMaterializedBodyPolicy`,
`PostgresColumnBound`, and `PostgresViewBoundMode` for the rule options. `columnBounds` maps catalog
`table.column` names to a required nonempty `reason`; CLI and Node use the same
catalog validation and caller-bounded key proof. No separate runtime export is
needed.

`materializedBodies` excludes existence and known correlated scalar probes.
Pool-building IN/array/derived bodies and explicit MATERIALIZED CTEs remain
independent obligations. A consumer of an existing materialized pool retains
that source obligation without repeating it; new unbounded scans still report.

`collectDiagnostics(operation, ...args)` is the asynchronous equivalent of CLI
`--verbose-timings`. Pass a public operation name and its normal arguments:

```js
const { collectDiagnostics } = require("no-mistakes");
const { result, diagnostics } = await collectDiagnostics("check", {
  root: process.cwd(),
});
```

`result` is the operation's ordinary result, including existing JavaScript
postprocessing. `diagnostics` is an exported `VerboseDiagnostics` report with
`elapsedMs`, `timings` (`DiagnosticTiming[]`), `work`, `phases`
(`PhaseResourceDiagnostic[]`), `hotFiles` (`HotFileDiagnostic[]`), and optional
`peakRssKiB`. `DiagnosticsResult<T>`, `DiagnosticFilePass`, and
`DiagnosticOperations` and `DiagnosticArguments<K>` are also exported. The selector
map preserves each operation's required or optional argument tuple and asynchronous
result type.
For a union-valued selector, arguments must satisfy every possible operation;
otherwise narrow the selector before calling. For example, `"check" | "dependents"`
requires options with `files`, even though a literal `"check"` needs no options.
Selectors cover root analysis APIs, including `analyzeProject`, SQL extraction
and parsing, and `lockStatus`. Utilities `version`, `diagnosticsSnapshot`,
`createWorkflowTopologyIndex`, and `writePlanningImpactArtifacts`, the collector
itself, and raw Markdown helpers `testsComment` and `testsGraphMermaid` are
excluded.

Phase records contain `label`, `wallMs`, user/system CPU milliseconds
(`userMs`, `systemMs`), `utilization` as CPU/wall time, process-lifetime peak RSS
(`peakRssKiB`) and its increase (`peakRssDeltaKiB`). Each label appears once,
in first-appearance order. Repeated eligible spans sum wall/user/system time;
utilization divides total CPU by total wall time. Peak RSS takes the maximum
phase-end sample and its delta sums the individual nonnegative span increases.
Resource sampling is Unix only; unsupported or failed samples have no resource
record. Overlapping independent serial spans omit both resource records;
intentional serial envelopes still cover their parallel workers. Nested and
parallel spans retain elapsed timings without CPU/RSS attribution. Hot-file records contain `pass`, `path`, `durationMs` and `units`,
bounded to ten per pass. File `parse` covers parsing and requested fact extraction;
value-flow extraction/finalization cover collector setup through returned facts.
All file durations are elapsed milliseconds, including annotation roots. See [diagnostic semantics](cli/diagnostics.md) for pass
units, ranking, RSS caveats and overlapping durations.

CPU/RSS samples cover the whole Node process, including concurrent work in
other requests. Use an otherwise idle process for phase attribution.

The native analysis runs on libuv async tasks and collection writes no stderr.
Each collector call owns its observer; ordinary calls remain uninstrumented
unless their existing timing or live-progress options are enabled. Reports are
returned to the caller without a completed-request registry. If observation
started before rejection, the error carries optional `.diagnostics`
(`DiagnosticsError`); invalid controls can reject before a report exists.
Existing `timings` and `diagnosticsId` behavior remains available unchanged.

`check({ timings: true })` returns ordered `timings` entries with `label`, `durationMs`, and `parallel`. Rule entries follow their phase, slowest first; combined passes are labeled accordingly. Parallel durations overlap.

Generated PostgreSQL catalogs include optional `functions[signature].returnsSet` evidence. `check()` uses it for `postgres-bounded-statements` LIMIT proofs; when metadata is absent, a parseable catalog `CREATE FUNCTION ... RETURNS type` declaration can prove scalar cardinality. Unknown, malformed and set-returning declarations stay conservative, matching the CLI.

`github-actions-max-inline-script-lines` is available through asynchronous `check()` and
`analyzeProject()` using the same configuration and `RuleFinding` report as CLI `check`.
Its positive integer `maxLines` option defaults to 10 and counts non-blank YAML
script-body lines, joining backslash continuations, in workflows and composite actions. See the
[rule documentation](rules/github-actions-max-inline-script-lines.md).

### Source-level merged configuration

```ts
import { resolveConfig, type DeclaredConfig, type DeclaredRule } from "no-mistakes";

const config: DeclaredConfig = await resolveConfig({ root: ".", view: "declared" });
const policy: DeclaredRule | undefined = config.rules?.find(
  (entry) => entry.rule === "rust-max-lines-per-file" && entry.name === "source-policy",
);
```

The declared view returns the source-level merged object directly, with canonical
aliases and no defaults, path expansion, or inherited rule materialization.
No config returns `{}`. The root file comes first; fragment lists concatenate in
sorted relative-path order. That order is stable and part of the API contract.
Explicit `config` selects one file and excludes fragments. Relative path strings
remain relative to the repository root.

`ConfigResolveOptions`, `ConfigResolveView`, `DeclaredConfig`, `DeclaredRule`,
`DeclaredConfigValue`, and `DeclaredConfigObject` are exported types. Rule options
use the recursive value type because authored options can be objects, arrays,
scalars, or null. Omitting `view`, or passing `view: "resolved"`, retains the
existing `ResolvedConfig` return shape. Readers that parse YAML or match its raw
text should use the declared view before splitting configuration; CI path filters
that name the root config must also cover `.no-mistakes/**`.

Allow-entry hygiene uses the existing asynchronous `check()` → `checkJson`
binding and `analyzeProject()` check-report path. New `unused-allow-entry`
results are `RuleFinding` objects in `CheckReport.advisories`: `file`/`line`
locate the config entry, `import` names the checked rule, and `target` carries
the selector. No runtime export or report field is added. Partial or uncertain
selector coverage stays silent; established rule-specific stale outputs retain
their existing shape and severity. See [allow-entry hygiene](configuration/suppressions.md#allow-entry-hygiene).

[`github-actions-local-action-checkout`](rules/github-actions-local-action-checkout.md)
uses the existing asynchronous `check()` and `analyzeProject()` check report
paths with the same configuration and ordered per-job checkout analysis as CLI
`check`. Findings use the existing `RuleFinding` shape: `rule` identifies the
rule, `file`/`line` locate the local action's `uses:` line, and `target` identifies
the `job#step` (one-based step position). No new runtime
export, option type, or report field is needed.

The configured [`vitest-project-policy`](rules/vitest-project-policy.md) rule
runs through `check()` and `analyzeProject()` check queries using the existing
async `checkJson` binding and JS facade. Exported `VitestProjectPolicyOptions`,
`VitestDeclaredProjectPolicy`, `VitestPropertyLiteralKind`, `VitestPropertyParity`,
and `VitestPropertyLiteral` describe its explicit config and helper path/glob
inputs, effective required/forbidden properties, parity pairs, and `declared`
literal restrictions. Helper globs select the prepared visible-file inventory.
`declared.forbiddenValues` and `declared.forbidLiteralKinds` inspect every object
literal in config and helpers, including unimported helpers; computed expressions
remain allowed without constant propagation. Declared-only policies do not
report dynamic projects as incomplete. Selected config/helper sources that cannot
be read or parsed, and unresolved effective-policy values, are returned as
`CheckReport.incomplete` (`CheckFailure[]`). Application include/exclude filters
and source suppressions apply to declaration diagnostics.

PostgreSQL executor options accept `extraExecutorNames` in the asynchronous
Node APIs, including `extractExecutedPostgresSql`. Each also accepts
`excludeExecutorNames: string[]`, applied last to remove names from the
replacement list or defaults plus extras.
`executorNames` replaces `[query, read, write]`; `extraExecutorNames` extends
those defaults, including an empty extension with no `importSpecifier`.
Passing both is an error. This also applies to the PostgreSQL
rule option interfaces and config passed to `check()`; its configuration
warnings identify omitted default executors imported and called in scoped
files. Excluded defaults suppress those warnings; exclusions that are not
default names or are never imported from the configured module and called in
scoped files produce unused-entry warnings. See [executor selection](configuration/rules.md#postgresql-executor-selection).

The opt-in [`postgres-lock-written-rows-only`](rules/postgres-lock-written-rows-only.md)
rule runs through the existing asynchronous `check()` API and the same rule
configuration as `no-mistakes check`; it adds no runtime export. Its public
contracts are `PostgresLockWrittenRowsOnlyOptions`,
`PostgresLockWrittenRowsCheck`, `PostgresLockWrittenRowsSeverity`,
`PostgresLockWrittenRowsAllow`, and `PostgresLockWrittenRowsWriteSet`.
`PostgresLockWrittenRowsOnlyOptions` includes `executorNames`,
`extraExecutorNames`, and `excludeExecutorNames` with the selection contract
above. Per-check severity defaults to `advise`. Unresolved SQL or transaction
membership follows the configured unanalyzable policy.

`PostgresLockWrittenRowsOnlyOptions.executorOptionProperties` selects static inline
executor-routing options (for example `{ query: tx }`); it defaults to `[]`.
Configured factory bindings support `const`, `using`, and `await using`.


The [`postgres-column-check-constraints`](rules/postgres-column-check-constraints.md)
catalog rule maps `check` to the existing async `check()` runtime export and
`analyzeProject()` check reports; it adds no runtime export. Named option types
are exported from `no-mistakes`; see [catalog checks](node-api/postgres-catalog.md).
