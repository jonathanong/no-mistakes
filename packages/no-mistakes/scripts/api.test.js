const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const packageRoot = join(__dirname, "..");
const addonPath = join(packageRoot, "bin", "no-mistakes.node");
const indexPath = join(packageRoot, "index.js");

test("programmatic API proxies object options through async native addon calls", async () => {
  const previous = require.extensions[".node"];
  delete require.cache[require.resolve(indexPath)];

  require.extensions[".node"] = (module, filename) => {
    assert.equal(filename, addonPath);
    module.exports = {
      dependenciesJson: async (json) =>
        JSON.stringify({ command: "dependencies", options: JSON.parse(json) }),
      dependentsJson: async (json) =>
        JSON.stringify({ command: "dependents", options: JSON.parse(json) }),
      relatedJson: async (json) =>
        JSON.stringify({ command: "related", options: JSON.parse(json) }),
      analyzeProjectJson: async (json) =>
        JSON.stringify({ command: "analyzeProject", options: JSON.parse(json) }),
      symbolsJson: async (json) =>
        JSON.stringify({ command: "symbols", options: JSON.parse(json) }),
      importersJson: async (json) =>
        JSON.stringify({ command: "importers", options: JSON.parse(json) }),
      exportsOfJson: async (json) =>
        JSON.stringify({ command: "exportsOf", options: JSON.parse(json) }),
      deadExportsJson: async (json) =>
        JSON.stringify({ command: "deadExports", options: JSON.parse(json) }),
      callSitesJson: async (json) =>
        JSON.stringify({ command: "callSites", options: JSON.parse(json) }),
      resolveCheckJson: async (json) =>
        JSON.stringify({ command: "resolveCheck", options: JSON.parse(json) }),
      fetchesJson: async (json) =>
        JSON.stringify({ command: "fetches", options: JSON.parse(json) }),
      checkJson: async (json) => JSON.stringify({ command: "check", options: JSON.parse(json) }),
      testsPlanJson: async (json) =>
        JSON.stringify({ command: "testsPlan", options: JSON.parse(json) }),
      testsWhyJson: async (json) =>
        JSON.stringify({ command: "testsWhy", options: JSON.parse(json) }),
      testsCommentMarkdown: async (json) =>
        `comment:${JSON.parse(json).plan || JSON.parse(json).planJson?.selected_tests?.length}`,
      testsGraphJson: async (json) =>
        JSON.stringify({ command: "testsGraph", options: JSON.parse(json) }),
      testsGraphMermaid: async (json) =>
        `graph:${JSON.parse(json).plan || JSON.parse(json).planJson?.selected_tests?.length}`,
      playwrightCheckJson: async (json) =>
        JSON.stringify({ command: "playwrightCheck", options: JSON.parse(json) }),
      playwrightEdgesJson: async (json) =>
        JSON.stringify({ command: "playwrightEdges", options: JSON.parse(json) }),
      playwrightRelatedJson: async (json) =>
        JSON.stringify({ command: "playwrightRelated", options: JSON.parse(json) }),
      playwrightTestsJson: async (json) =>
        JSON.stringify({ command: "playwrightTests", options: JSON.parse(json) }),
      queuesJson: async (json) => JSON.stringify({ command: "queues", options: JSON.parse(json) }),
      queueEdgesJson: async (json) =>
        JSON.stringify({ command: "queueEdges", options: JSON.parse(json) }),
      queueRelatedJson: async (json) =>
        JSON.stringify({ command: "queueRelated", options: JSON.parse(json) }),
      queueCheckJson: async (json) =>
        JSON.stringify({ command: "queueCheck", options: JSON.parse(json) }),
      serverRoutesJson: async (json) =>
        JSON.stringify({ command: "serverRoutes", options: JSON.parse(json) }),
      serverRouteListJson: async (json) =>
        JSON.stringify({ command: "serverRouteList", options: JSON.parse(json) }),
      serverRouteEdgesJson: async (json) =>
        JSON.stringify({ command: "serverRouteEdges", options: JSON.parse(json) }),
      serverRouteRelatedJson: async (json) =>
        JSON.stringify({ command: "serverRouteRelated", options: JSON.parse(json) }),
      reactAnalyzeJson: async (json) =>
        JSON.stringify({ command: "reactAnalyze", options: JSON.parse(json) }),
      reactCheckJson: async (json) =>
        JSON.stringify({ command: "reactCheck", options: JSON.parse(json) }),
      reactUsagesJson: async (json) =>
        JSON.stringify({ command: "reactUsages", options: JSON.parse(json) }),
      infraResourceRefsJson: async (json) =>
        JSON.stringify({ command: "infraResourceRefs", options: JSON.parse(json) }),
      infraOutputsJson: async (json) =>
        JSON.stringify({ command: "infraOutputs", options: JSON.parse(json) }),
      infraTestForJson: async (json) =>
        JSON.stringify({ command: "infraTestFor", options: JSON.parse(json) }),
      swiftImportersJson: async (json) =>
        JSON.stringify({ command: "swiftImporters", options: JSON.parse(json) }),
      swiftTestTargetsJson: async (json) =>
        JSON.stringify({ command: "swiftTestTargets", options: JSON.parse(json) }),
      version: async () => "1.2.3",
    };
  };

  try {
    const api = require(indexPath);
    assert.deepEqual(await api.dependencies({ files: ["a.mts"] }), {
      command: "dependencies",
      options: { files: ["a.mts"] },
    });
    assert.equal((await api.dependents({ files: ["b.mts"] })).command, "dependents");
    assert.equal((await api.related({ files: ["c.mts"] })).command, "related");
    assert.equal(
      (await api.analyzeProject({ reports: [{ type: "dependencies", files: ["a.mts"] }] })).command,
      "analyzeProject",
    );
    assert.equal(
      (await api.symbols({ files: ["d.mts"], include: "both" })).options.include,
      "both",
    );
    assert.equal(
      (
        await api.symbols({
          files: ["d.mts"],
          mode: "signature-impact",
          symbol: "handler",
        })
      ).options.mode,
      "signature-impact",
    );
    assert.deepEqual(await api.importers({ file: "a.ts", tests: true }), {
      command: "importers",
      options: { file: "a.ts", tests: true },
    });
    assert.equal((await api.exportsOf({ file: "a.ts" })).command, "exportsOf");
    assert.equal((await api.deadExports({ file: "a.ts", names: ["foo"] })).options.names[0], "foo");
    assert.equal(
      (await api.callSites({ file: "a.ts", exportName: "foo" })).options.exportName,
      "foo",
    );
    assert.equal((await api.resolveCheck({ file: "a.ts" })).command, "resolveCheck");
    assert.equal((await api.fetches({ targets: ["/users"] })).command, "fetches");
    assert.equal((await api.check({ tsconfig: "tsconfig.json" })).command, "check");
    assert.deepEqual(
      (await api.testsPlan({ framework: "swift", globalConfigFallback: false })).options,
      { framework: "swift", globalConfigFallback: false },
    );
    assert.equal((await api.testsWhy({ test: "source.test.ts" })).command, "testsWhy");
    assert.equal(await api.testsComment({ plan: "plan.json" }), "comment:plan.json");
    assert.equal(
      (await api.testsGraph({ planJson: { selected_tests: [] } })).command,
      "testsGraph",
    );
    assert.equal(await api.testsGraphMermaid({ planJson: { selected_tests: [] } }), "graph:0");
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
    assert.equal(
      (await api.serverRouteEdges({ roots: ["routes.ts"] })).command,
      "serverRouteEdges",
    );
    assert.equal(
      (await api.serverRouteRelated({ roots: ["routes.ts"] })).command,
      "serverRouteRelated",
    );
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
    assert.equal((await api.swiftImporters({ file: "Sources/A.swift" })).command, "swiftImporters");
    assert.equal(
      (await api.swiftTestTargets({ file: "Sources/A.swift" })).command,
      "swiftTestTargets",
    );
    assert.equal(await api.version(), "1.2.3");
  } finally {
    delete require.cache[require.resolve(indexPath)];
    if (previous) {
      require.extensions[".node"] = previous;
    } else {
      delete require.extensions[".node"];
    }
  }
});

test("analyzeProject declarations mirror report-specific runtime requirements", () => {
  const declarations = readFileSync(join(packageRoot, "traversal-types.d.ts"), "utf8");
  assert.match(declarations, /export type SymbolsSignatureImpactOptions = SymbolsOptions & \{/);
  assert.match(
    readFileSync(join(packageRoot, "index.d.ts"), "utf8"),
    /export function symbols\(options: SymbolsOptions\): Promise<SymbolsResult \| SignatureImpactResult>;/,
  );
  assert.match(declarations, /mode: "signature-impact";\n  symbol: string;/);
  assert.match(
    declarations,
    /type: "symbols"; id\?: string } & \(SymbolsListOptions \| SymbolsSignatureImpactOptions\)/,
  );
  assert.match(
    declarations,
    /type BatchedQueueRelatedOptions = BatchedProjectOptions & \{ files: string\[\] \}/,
  );
  assert.match(
    declarations,
    /type BatchedServerRouteRelatedOptions = BatchedProjectOptions &\n  \(\{ files: string\[\] \} \| \{ roots: string\[\] \}\)/,
  );
  assert.match(
    declarations,
    /type: "playwrightRelated"; id\?: string } & Omit<PlaywrightRelatedOptions,/,
  );
});
