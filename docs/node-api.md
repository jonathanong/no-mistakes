# Node/N-API Guide

The `no-mistakes` npm package exposes async functions backed by the same Rust
analysis as the CLI. Use it when an agent or tool needs repeated structured
queries without subprocess overhead.

```js
const {
  analyzeProject,
  dependents,
  importUsages,
  symbols,
  testsPlan,
  validateMermaidMarkdown,
} = require("no-mistakes");

(async () => {
  const impact = await dependents({
    root: process.cwd(),
    files: ["src/api.mts#handler"],
    tests: ["vitest", "dotnet", "swift", "python", "go", "cargo", "rails", "php", "jest"],
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
```

## CLI Mapping

| CLI | Node API |
| --- | --- |
| `dependencies` | `dependencies(options)` |
| `dependents` | `dependents(options)` |
| `related` | `related(options)` |
| `symbols` | `symbols(options)` |
| `import-usages` | `importUsages(options)` |
| `importers` | `importers(options)` |
| `exports-of` | `exportsOf(options)` |
| `dead-exports` | `deadExports(options)` |
| `call-sites` | `callSites(options)` |
| `resolve-check` | `resolveCheck(options)` |
| `fetches` | `fetches(options)` |
| `flow` | `flow(options)` |
| `check` | `check(options)` |
| `config resolve` | `resolveConfig(options)` |
| `data-pw` | `dataPw(options)` |
| `effects` | `effects(options)` |
| `rsc-callers` | `rscCallers(options)` |
| `registry-extension` | `registryExtension(options)` |
| `tests plan` | `testsPlan(options)`; `framework` accepts `vitest`, `playwright`, `dotnet`, `swift`, `python`, `go`, `cargo`, `rails`, `php`, or `jest`. Import `TestPlanFramework` for that union instead of indexing `TestExecutionTarget['runner']` |
| `tests targets` | `testsTargets(options)` |
| `tests impact` | `testsImpact(options)` |
| `tests why` | `testsWhy(options)` |
| `tests comment` | `testsComment(options)` |
| `tests graph` | `testsGraph(options)` or `testsGraphMermaid(options)` |
| `playwright check\|edges\|related\|tests` | `playwrightCheck`, `playwrightEdges`, `playwrightRelated`, `playwrightTests` |
| `queues edges\|related\|check` | `queueEdges`, `queueRelated`, `queueCheck` |
| `server routes\|edges\|related\|contracts` | `serverRouteList`, `serverRouteEdges`, `serverRouteRelated`, `serverContracts` |
| `react analyze\|check\|usages` | `reactAnalyze`, `reactCheck`, `reactUsages` |
| `infra resource-refs\|outputs\|test-for` | `infraResourceRefs`, `infraOutputs`, `infraTestFor` |
| `swift importers\|test-targets` | `swiftImporters`, `swiftTestTargets` |
| `lockfile diff` | `lockfileDiff(options)` |
| `ci impact` | `ciImpact(options)` |
| `ci env` | `ciEnv(options)` |
| `ci topology` | `ciTopology(options)` |
| `impacted-checks` | `impactedChecks(options)` |

The following inventory is the complete runtime export surface. Keeping this
list exhaustive makes a newly added function visible to agents even when it
does not have a one-to-one CLI command:

| Runtime export | API |
| --- | --- |
| `createWorkflowTopologyIndex` | `createWorkflowTopologyIndex(topology)` |
| `version` | `version()` |
| `analyzeProject` | `analyzeProject(options)` |
| `callSites` | `callSites(options)` |
| `check` | `check(options)` |
| `ciEnv` | `ciEnv(options)` |
| `ciImpact` | `ciImpact(options)` |
| `ciTopology` | `ciTopology(options)` |
| `dataPw` | `dataPw(options)` |
| `deadExports` | `deadExports(options)` |
| `dependencies` | `dependencies(options)` |
| `dependents` | `dependents(options)` |
| `effects` | `effects(options)` |
| `exportsOf` | `exportsOf(options)` |
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
`runner_args` already contain the correct flag. This includes configured and
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

`testsPlan(options)` returns `changed_files`, the sorted, deduplicated
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
targets, and returns a `direct-test-owner` group with `fallback_triggered:
false`. `limitPercent`, `limitFiles`, and `globalConfigFallback` conflict with
this option. `entrypoints` also conflicts with it: direct-owner selection is
bounded to changed files and one reverse canonical graph edge, so use
`testsImpact()` for explicit entrypoint traversal. The returned warnings retain
canonical graph diagnostics for dynamic resource calls in changed files, so
incomplete reverse ownership is visible to API consumers.

The TypeScript declaration models this as a discriminated option: direct-owner
plans require `framework`, while ordinary plans omit `directTestOwner` or set it
to `false`.

`testsPlan(options)` returns `fallback_triggered` and `fallback_reason` when a
`dotnet` or `swift` plan has to fall back from native graph tracing to
framework-scoped discovered tests. Vitest plans also use this surface for a
dynamic or unresolved `setupFiles`/`globalSetup` declaration: the result is
bounded to its known project owner when possible. Its helper closure follows
ordinary static imports/re-exports and literal CommonJS `require(...)` or
`require.resolve(...)` dependencies, retaining edits and deletions as owner
triggers; computed or non-literal forms are not followed. Resolved setup paths
use `via: ["vitest-setup"]` and may add `via_details`, an optional array aligned
with `via` whose setup edge detail is `{ type: "vitest-setup", field:
"setupFiles" | "globalSetup" }`.

`testsWhy()` and `testsGraph()` expose the same optional structured `detail`,
and the Mermaid graph renders the Vitest field in the edge label. The optional
fields preserve compatibility with previously saved plan JSON and are absent
for ordinary edges.

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
`fallback_triggered` false. Semantic `.no-mistakes.yml`/`.yaml` invalidation is
also identical for revision and inline-diff inputs.

`testsPlan`, `testsImpact`, `testsWhy`, and `testsGraph` expose resource-edge
provenance without a separate API: plan reasons use optional edge-aligned
`via_details`, why steps use optional `detail`, and graph JSON edges use
optional `detail`. Details are `{ type: "resource", consumer_file,
call_sites: [{ call_kind, line }] }` for literal runtime filesystem edges or
`{ type: "vitest-setup", field: "setupFiles" | "globalSetup" }` for setup
edges.

`check(options)` returns the same structured check report as CLI JSON,
including `warnings: string[]` for configured checks that could not run.

`resolveConfig(options)` returns the same JSON as `config resolve`: frontend
apps, Playwright coverage gates, Vitest `vitestFullSuiteTriggers`, and the
additive `fullSuiteTriggers` array keyed by `TestPlanFramework`. Existing
`vitestFullSuiteTriggers` contents stay unchanged.

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
reports may override `root`, `tsconfig`, and `config`; `reactUsages` accepts
`target`, `targets`, `include`, and scope options; and `check` may override
`root`, `tsconfig`, and `config`. Lightweight queries (`importers`, `exportsOf`,
`deadExports`, `callSites`, `resolveCheck`), `fetches`, test-plan reports,
`lockfileDiff`, CI/infra/swift reports, `impactedChecks`, and
`validateMermaidMarkdown` are also valid `reports[].type` values. They inherit
the request `root`/`tsconfig`/`config` and dispatch through the dedicated Node
APIs. Reports with the same effective scope share
one request-scoped in-memory dataset. Sources, parsed metadata, and compact file
facts are reused; each normalized graph or symbol-index plan is built at most
once for its file universe. Distinct effective scopes are prepared independently.

When a request omits `tsconfig`, TypeScript/JavaScript imports are resolved with
the config that owns each importing file. Dependency graph and query APIs plus
test planning use this behavior across referenced workspace projects. Set
`tsconfig` only to force that one config for the entire request; this preserves
the previous single-config behavior for debugging and compatibility.

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
`lockTimeout`. `jobs` sets the rayon worker count for that invocation;
omit it to leave the process pool unchanged, and pass `0` to use the CPU
count (matching CLI `--jobs 0`).

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

## Agent Defaults

- Pass `root` explicitly.
- Omit `tsconfig` to use automatic per-workspace resolution; pass it explicitly
  only to force one config for debugging or compatibility.
- Use `analyzeProject()` when several reports share the same root/config.
- Prefer structured API results over parsing human CLI output.
