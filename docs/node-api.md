# Node/N-API Guide

The `no-mistakes` npm package exposes async functions backed by the same Rust
analysis as the CLI. Use it when an agent or tool needs repeated structured
queries without subprocess overhead.

```js
const { analyzeProject, dependents, symbols, testsPlan } = require("no-mistakes");

(async () => {
  const impact = await dependents({
    root: process.cwd(),
    files: ["src/api.mts#handler"],
    tests: ["vitest"],
  });

  const report = await analyzeProject({
    root: process.cwd(),
    reports: [
      { type: "symbols", files: ["src/api.mts"], include: "both" },
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
| `fetches` | `fetches(options)` |
| `check` | `check(options)` |
| `tests plan` | `testsPlan(options)` |
| `tests impact` | `testsImpact(options)` |
| `tests why` | `testsWhy(options)` |
| `tests comment` | `testsComment(options)` |
| `tests graph` | `testsGraph(options)` or `testsGraphMermaid(options)` |
| `playwright check\|edges\|related\|tests` | `playwrightCheck`, `playwrightEdges`, `playwrightRelated`, `playwrightTests` |
| `queues edges\|related\|check` | `queueEdges`, `queueRelated`, `queueCheck` |
| `server routes\|edges\|related` | `serverRouteList`, `serverRouteEdges`, `serverRouteRelated` |
| `react analyze\|check` | `reactAnalyze`, `reactCheck` |

`check(options)` returns the same structured check report as CLI JSON,
including `warnings: string[]` for configured checks that could not run.

## Agent Defaults

- Pass `root` explicitly.
- Pass `tsconfig` explicitly in monorepos with package-local aliases.
- Use `analyzeProject()` when several reports share the same root/config.
- Prefer structured API results over parsing human CLI output.
