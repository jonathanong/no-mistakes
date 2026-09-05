# no-mistakes

Local AST graph for coding agents: impact, focused test plans, Playwright
coverage, and repository checks. Use it when grep would miss callers behind
aliases, when a change should not run every suite, or when a new page needs a
coverage gate. See [why it exists](../../docs/why.md).

The async N-API facade avoids subprocess parsing and reuses one prepared
analysis for related reports.

```bash
npm install --save-dev no-mistakes
npx no-mistakes dependencies src/main.mts --json
npx no-mistakes dependents src/utils.mts --json
npx no-mistakes symbols src/utils.mts --json
npx no-mistakes import-usages --root . --filter 'src/**' --json
npx no-mistakes check --json
npx no-mistakes planning-impact --changed-files /private/run/changed-files.txt --output-dir /private/run
```

`planning-impact` is supplied by the npm package, not the Cargo-installed
binary. It writes the private four-report artifact contract from one prepared
analysis. See the [CLI reference](../../docs/cli/planning-impact.md).

Programmatic Node usage loads the same Rust analysis through N-API:

```js
const { analyzeProject } = require("no-mistakes");

const report = await analyzeProject({
  root: process.cwd(),
  reports: [
    { type: "dependents", files: ["src/api.mts"] },
    { type: "symbols", files: ["src/api.mts"], include: "both" },
    {
      type: "testsPlan",
      framework: "vitest",
      changedFiles: ["src/api.mts"],
    },
  ],
});
```

Use `analyzeProject()` when reports share a root and configuration. Dedicated
functions remain convenient for a single query:

````js
const {
  dependencies,
  dependents,
  check,
  fetches,
  importUsages,
  flow,
  playwrightRelated,
  symbols,
  testsPlan,
  testsTargets,
  queueEdges,
  queueRelated,
  queueCheck,
  serverRouteList,
  serverRouteEdges,
  serverRouteRelated,
  serverContracts,
  reactAnalyze,
  reactCheck,
  validateMermaidMarkdown,
} = require("no-mistakes");

(async () => {
  const deps = await dependencies({
    root: process.cwd(),
    files: ["src/main.mts"],
    relationships: ["import"],
  });
  const tests = await dependents({
    root: process.cwd(),
    files: ["src/utils.mts"],
    tests: ["vitest"],
  });
  const symbolFacts = await symbols({
    root: process.cwd(),
    files: ["src/utils.mts"],
    include: "both",
  });
  const imports = await importUsages({
    root: process.cwd(),
    filters: ["src/**"],
  });
  const signatureImpact = await symbols({
    root: process.cwd(),
    files: ["src/utils.mts"],
    mode: "signature-impact",
    symbol: "parseDate",
  });
  const plan = await testsPlan({
    root: process.cwd(),
    framework: "vitest", // also jest, python, go, cargo, rails, php, java, kotlin, elixir, dart, playwright, dotnet, swift
    changedFiles: ["src/utils.mts"],
  });
  // Complete changed-file inventory, including paths that selected no tests.
  console.log(plan.changedFiles);
  const targetCommands = await testsTargets({
    root: process.cwd(),
    framework: "vitest",
    files: ["src/utils.test.mts"],
  });
  const projectCheck = await check({
    root: process.cwd(),
    // Path to tsconfig.json for alias resolution; searched upward if omitted.
    // In monorepos, pass the workspace-scoped tsconfig (e.g. "web/tsconfig.json").
    tsconfig: "tsconfig.json",
  });
  const mermaid = await validateMermaidMarkdown({
    content: "```mermaid\nflowchart LR\n  A --> B\n```",
    file: "docs/design.md",
  });
  const localFlow = await flow({
    root: process.cwd(),
    target: "src/utils.mts#parseDate",
    direction: "dependents",
    depth: 1,
  });
  const coveredByPlaywright = await playwrightRelated({
    root: process.cwd(),
    files: ["web/app/users/page.tsx"],
  });

  const queueHops = await queueRelated({
    root: process.cwd(),
    files: ["src/jobs/enqueue.ts"],
    direction: "both",
  });
  const routeEdges = await serverRouteEdges({
    root: process.cwd(),
    roots: ["src/server.ts"],
  });
  const contracts = await serverContracts({
    root: process.cwd(),
    roots: ["src/server.ts"],
  });
  const components = await reactAnalyze({
    root: process.cwd(),
    targets: ["app/**/*.tsx"],
  });
})();
````

CLI and Node analyses share a per-user machine-wide lock. CLI flags
`--timeout`, `--lock-timeout`, and `--fail-on-lock` have Node equivalents
`timeout`, `lockTimeout`, and `failOnLock`. CLI timeouts default to 30 seconds;
Node/N-API omits both deadlines unless you set them. `0` disables either CLI
timeout, while `0` or `null` disables it in Node. While waiting, stderr reports
the holder pid and elapsed seconds. Waiting does not alter successful output,
and Node lock/timeout failures reject the returned Promise.

Dependency graph, query, and test-planning resolution is per workspace by
default: when `tsconfig` is omitted, each import uses the config that owns its
importing file, including referenced projects. This keeps conflicting package
aliases isolated while shared code can still select all importing tests. Pass
`tsconfig` explicitly to force one config for a whole invocation when debugging
or preserving a legacy single-config workflow.

Graph queries also support `relationships: ["workflow"]` for canonical local
GitHub Actions traversal: workflow file -> virtual job -> virtual step,
`needs`, local reusable workflows/actions, supported literal `run:` targets and
package scripts, and same-run artifact handoffs. Virtual IDs are
`workflow.yml#job:<job>` and `workflow.yml#job:<job>/step:<zero-based-index>`.
Use the precise `workflow-job`, `workflow-step`, `workflow-needs`,
`workflow-uses`, `workflow-run`, or `workflow-artifact` filters to select a
semantic while retaining its required structural bridge edges. The legacy `ci`
relationship stays separate: it covers only workflow-file-to-Rust-binary Cargo
invocations. Remote actions/workflows, `workflow_run`, dynamic shell commands,
and paths outside the tracked graph are intentionally excluded.

External `no-mistakes-*` executables on `PATH` can be invoked as subcommands.
For example, after installing `no-mistakes-scripts`:

```bash
npm install --save-dev no-mistakes-scripts
npx no-mistakes rust-no-inline-tests crates/*/src
npx no-mistakes rust-max-lines-per-file crates/*/src crates/*/tests
```

See the full documentation in [docs/](../../docs/README.md), the
[CLI command index](../../docs/cli/README.md), and the
[Node/N-API guide](../../docs/node-api.md). Agents can use the compact
[packaged skill](../../skills/no-mistakes/SKILL.md).
