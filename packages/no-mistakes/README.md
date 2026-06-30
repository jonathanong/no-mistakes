# no-mistakes

Unified static codebase intelligence CLI for TS/JS module graphs, symbols,
React traits, queue hops, server routes, and project checks.

```bash
npm install --save-dev no-mistakes
npx no-mistakes dependencies src/main.mts --json
npx no-mistakes dependents src/utils.mts --json
npx no-mistakes symbols src/utils.mts --json
npx no-mistakes import-usages --root . --filter 'src/**' --json
npx no-mistakes check --json
```

Programmatic Node usage loads the same Rust analysis through N-API:

```js
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
    framework: "vitest", // also supports "playwright", "dotnet", and "swift"
    changedFiles: ["src/utils.mts"],
  });
  const targetCommands = await testsTargets({
    root: process.cwd(),
    framework: "vitest",
    files: ["src/utils.test.mts"],
  });
  const projectCheck = await check({
    root: process.cwd(),
    tsconfig: "tsconfig.json",
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
```

External `no-mistakes-*` executables on `PATH` can be invoked as subcommands.
For example, after installing `no-mistakes-scripts`:

```bash
npm install --save-dev no-mistakes-scripts
npx no-mistakes rust-no-inline-tests crates/*/src
npx no-mistakes rust-max-lines-per-file crates/*/src crates/*/tests
```

See the full documentation in [docs/](../../docs/README.md), the
[CLI command index](../../docs/cli/README.md), and the
[Node/N-API guide](../../docs/node-api.md).
