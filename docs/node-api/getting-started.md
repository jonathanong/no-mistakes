# Getting started

The `no-mistakes` npm package exposes async functions backed by the same Rust
analysis as the CLI. Use it when an agent or tool needs repeated structured
queries without subprocess overhead.

`playwrightCheck` and `playwrightEdges` honor opt-in
[integration route coverage sources](../configuration/integration-route-coverage.md).
`resolveConfig` exposes those declarations as `routeCoverageSources`; the
exported `RouteCoverageSource`, `RouteCoverageHelper`, `RouteCoverageFramework`,
and `RouteCoverageAttribution` types describe the contract.

`analyzeProject()` report entries use the exported `AnalyzeProjectReport` and
`AnalyzeProjectReportType` names; import those instead of indexing
`AnalyzeProjectReportRequest['type']`.

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

[Node/N-API guide](../node-api.md)
