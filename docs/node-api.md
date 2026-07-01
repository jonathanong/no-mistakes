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
      { type: "symbols", files: ["src/api.mts"], include: "both" },
      { type: "symbols", files: ["src/api.mts"], mode: "signature-impact", symbol: "handler" },
      { type: "check" },
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
| `impacted-checks` | `impactedChecks(options)` |

`testsPlan(options)` returns `fallback_triggered` and `fallback_reason` when a
`dotnet` or `swift` plan has to fall back from native graph tracing to
framework-scoped discovered tests.

`check(options)` returns the same structured check report as CLI JSON,
including `warnings: string[]` for configured checks that could not run.

## Agent Defaults

- Pass `root` explicitly.
- Pass `tsconfig` explicitly in monorepos with package-local aliases.
- Use `analyzeProject()` when several reports share the same root/config.
- Prefer structured API results over parsing human CLI output.
