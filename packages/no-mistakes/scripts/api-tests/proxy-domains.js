const { assert, writeFileSync, mkdtempSync, join, resolve, tmpdir } = require("./context");

async function assertProxyDomains(api) {
  assert.equal((await api.fetches({ targets: ["/users"] })).command, "fetches");
  assert.deepEqual(
    await api.check({
      tsconfig: "tsconfig.json",
      includeSuppressed: true,
      includeRunnerConfigDeadlines: true,
    }),
    {
      command: "check",
      options: {
        tsconfig: "tsconfig.json",
        includeSuppressed: true,
        includeRunnerConfigDeadlines: true,
      },
    },
  );
  assert.equal((await api.resolveConfig({ root: "." })).command, "resolveConfig");
  assert.deepEqual(
    await api.validateMermaidMarkdown({ content: "diagram source", file: "docs/design.md" }),
    {
      command: "validateMermaidMarkdown",
      options: { content: "diagram source", file: "docs/design.md" },
    },
  );
  assert.deepEqual(
    (await api.testsPlan({ framework: "swift", globalConfigFallback: false })).options,
    { framework: "swift", globalConfigFallback: false },
  );
  assert.equal(
    (await api.testsTargets({ framework: "vitest", files: ["source.test.ts"] })).command,
    "testsTargets",
  );
  assert.equal((await api.testsWhy({ test: "source.test.ts" })).command, "testsWhy");
  assert.equal(await api.testsComment({ plan: "plan.json" }), "comment:plan.json");
  assert.equal((await api.testsGraph({ planJson: { selected_tests: [] } })).command, "testsGraph");
  assert.equal(await api.testsGraphMermaid({ planJson: { selected_tests: [] } }), "graph:0");
  const planDir = mkdtempSync(join(tmpdir(), "no-mistakes-plan-"));
  const planPath = join(planDir, "plan.json");
  writeFileSync(planPath, JSON.stringify({ selectedTests: [] }));
  assert.equal(await api.testsGraphMermaid({ plan: planPath }), "graph:0");
  assert.equal((await api.playwrightCheck({ root: "." })).command, "playwrightCheck");
  assert.equal((await api.playwrightEdges({ root: "." })).command, "playwrightEdges");
  assert.equal(
    (await api.playwrightRelated({ files: ["app/page.tsx"] })).command,
    "playwrightRelated",
  );
  assert.equal(
    (await api.playwrightTests({ files: ["tests/app.spec.ts"] })).command,
    "playwrightTests",
  );
  assert.equal((await api.queues({ root: "." })).command, "queues");
  assert.equal((await api.queueEdges({ files: ["queue.ts"] })).command, "queueEdges");
  assert.equal((await api.queueRelated({ files: ["queue.ts"] })).command, "queueRelated");
  assert.equal((await api.queueCheck({ root: "." })).command, "queueCheck");
  assert.equal((await api.serverRoutes({ root: "." })).command, "serverRoutes");
  assert.equal((await api.serverRouteList({ files: ["/api"] })).command, "serverRouteList");
  assert.equal((await api.serverRouteEdges({ roots: ["routes.ts"] })).command, "serverRouteEdges");
  assert.equal(
    (await api.serverRouteRelated({ roots: ["routes.ts"] })).command,
    "serverRouteRelated",
  );
  assert.equal((await api.parsePostgresSql({ sql: "SELECT 1" })).command, "parsePostgresSql");
  assert.deepEqual((await api.parsePostgresSql([{ sql: "SELECT 1" }])).options, [
    { sql: "SELECT 1" },
  ]);
  assert.deepEqual(
    (await api.extractExecutedPostgresSql({ sources: [], importSpecifier: "@db" })).options,
    { sources: [], importSpecifier: "@db" },
  );
  const importAware = {
    sources: [],
    importSpecifier: "@db",
    resolveImports: true,
    resolvedImports: [{ fromFileName: "a.ts", specifier: "@sql", toFileName: "b.ts" }],
  };
  assert.deepEqual((await api.extractExecutedPostgresSql(importAware)).options, importAware);
  assert.deepEqual(
    await api.extractExecutedPostgresSql({ ...importAware, includeDiagnostics: true }),
    {
      statements: [],
      undeclaredExecutorTypes: [{ type: "PoolClient", module: "@db", count: 1 }],
    },
  );
  const noDiagnostics = { ...importAware, includeDiagnostics: false };
  assert.deepEqual((await api.extractExecutedPostgresSql(noDiagnostics)).options, noDiagnostics);
  assert.equal((await api.serverContracts({ root: "." })).command, "serverContracts");
  assert.equal((await api.flow({ target: "src/api.ts#handler" })).command, "flow");
  assert.equal((await api.reactAnalyze({ targets: ["*.tsx"] })).command, "reactAnalyze");
  assert.equal((await api.reactCheck({ assertNoFetch: true })).command, "reactCheck");
  assert.equal((await api.reactUsages({ target: "a.tsx#Button" })).command, "reactUsages");
  assert.equal(
    (await api.infraResourceRefs({ address: "aws_lb.web" })).command,
    "infraResourceRefs",
  );
  assert.equal(
    (await api.infraOutputs({ moduleDir: "infra/modules/net" })).command,
    "infraOutputs",
  );
  assert.equal((await api.infraTestFor({ tfFile: "infra/main.tf" })).command, "infraTestFor");
  assert.equal(
    (await api.generatePostgresCatalog({ connectionEnv: "DATABASE_URL", schema: "public" }))
      .command,
    "generatePostgresCatalog",
  );
  assert.equal(
    (
      await api.generatePostgresCatalog({
        connectionEnv: "DATABASE_URL",
        schema: "public",
        coverage: "ordering",
      })
    ).options.coverage,
    "ordering",
  );
  assert.equal((await api.swiftImporters({ file: "Sources/A.swift" })).command, "swiftImporters");
  assert.equal(
    (await api.swiftTestTargets({ file: "Sources/A.swift" })).command,
    "swiftTestTargets",
  );
  assert.equal((await api.ciTopology({ workflows: ["ci.yml"] })).options.workflows[0], "ci.yml");
  assert.equal(
    (await api.ciTopology({ workflows: ["ci.yml"] })).options.root,
    resolve(process.cwd()),
  );
  assert.equal(
    (await api.ciTopology({ workflows: ["deploy.yml"] })).options.workflows[0],
    "deploy.yml",
  );
  assert.equal(
    (await api.ciTopologyImpact({ base: "base", head: "head", entryWorkflow: "ci.yml" })).options
      .entryWorkflow,
    "ci.yml",
  );
  const topology = await api.ciTopology({ workflows: ["ci.yml"] });
  topology.options.workflows[0] = "mutated.yml";
  assert.equal((await api.ciTopology({ workflows: ["ci.yml"] })).options.workflows[0], "ci.yml");
  assert.equal(await api.version(), "1.2.3");
}

module.exports = { assertProxyDomains };
