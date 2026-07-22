# Node/N-API Guide

The `no-mistakes` npm package exposes async functions backed by the same Rust
analysis as the CLI. Use it when an agent or tool needs repeated structured
queries without subprocess overhead.

```js
const { analyzeProject, dependents, importUsages, symbols, testsPlan } = require("no-mistakes");

(async () => {
  const impact = await dependents({
    root: process.cwd(),
    files: ["src/api.mts#handler"],
    tests: ["vitest", "dotnet", "swift"],
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

  console.log({ impact, report });
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
| `fetches` | `fetches(options)` |
| `flow` | `flow(options)` |
| `check` | `check(options)` |
| `tests plan` | `testsPlan(options)`; `framework` accepts `vitest`, `playwright`, `dotnet`, or `swift` |
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

The Playwright APIs load the same selector-wrapper configuration as the CLI.
Configured wrapper calls therefore appear in `playwrightEdges()` and
`analyzeProject()` through the existing selector-edge JSON shape; no separate
Node option or result type is required.

`testsPlan(options)` returns `fallback_triggered` and `fallback_reason` when a
`dotnet` or `swift` plan has to fall back from native graph tracing to
framework-scoped discovered tests.

The API uses the same target-scoped `fullSuiteTriggers.projects` behavior as the
CLI. A `{ paths, targets }` match selects only tests owned by those runner
projects, emits `configured-trigger` reasons and execution targets, and leaves
`fallback_triggered` false. Semantic `.no-mistakes.yml`/`.yaml` invalidation is
also identical for revision and inline-diff inputs.

`check(options)` returns the same structured check report as CLI JSON,
including `warnings: string[]` for configured checks that could not run.

Each `analyzeProject()` report may use its report-specific options. Graph
reports may override `root`, `tsconfig`, and `config`; `reactUsages` accepts
`target`, `targets`, `include`, and scope options; and `check` may override
`root`, `tsconfig`, and `config`. Reports with the same effective scope share
one request-scoped in-memory dataset. Sources, parsed metadata, and compact file
facts are reused; each normalized graph or symbol-index plan is built at most
once for its file universe. Distinct effective scopes are prepared independently.

`ciTopology(options)` returns the same schema-v1 `WorkflowTopology` JSON as
`ci topology --format json` — it never throws on diagnostics (unlike the CLI,
which exits non-zero and prints nothing when any diagnostic is an error);
callers inspect the returned `diagnostics` array themselves.
`createWorkflowTopologyIndex(topology)` builds a frozen, sorted query index
(`directUpstreamJobIds`, `transitiveCalleeWorkflowPaths`,
`artifactConsumersForProducerJob`, etc.) over that result — it is pure JS,
runs entirely client-side, and never crosses the N-API boundary itself:

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
  timings: true,
});

// report.timings: [{ phase: "prepare", duration_ms: 12 }, ...]
```

Timing entries use stable phase identifiers and fractional-millisecond
durations. The lazy `graph` phase is present only when dependency analysis is
needed. The property is omitted by default. Unlike CLI `--timings`, Node timing
collection does not print progress to stderr.

## Invocation Lock And Timeouts

Every async analysis function except `version()` accepts these common options:

```ts
interface InvocationOptions {
  timeout?: number | null;
  lockTimeout?: number | null;
  failOnLock?: boolean;
}
```

Durations are non-negative integer seconds. `timeout` limits command execution
after the lock is acquired, while `lockTimeout` limits only the lock wait. Both
default to 30 seconds; `0` or `null` disables the corresponding timeout.
`failOnLock: true` fails immediately on contention and overrides
`lockTimeout`.

The lock is shared by CLI and Node/N-API analyses for the current OS user across
all repositories. Waiting is silent, successful return values keep their
existing shapes, and lock or timeout failures reject the returned Promise with
an actionable error. For `analyzeProject()`, put these options at the top level,
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
- Pass `tsconfig` explicitly in monorepos with package-local aliases.
- Use `analyzeProject()` when several reports share the same root/config.
- Prefer structured API results over parsing human CLI output.
