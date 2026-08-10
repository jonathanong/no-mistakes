const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { pathToFileURL } = require("node:url");

const packageRoot = join(__dirname, "..");
const addonPath = join(packageRoot, "bin", "no-mistakes.node");
const indexPath = join(packageRoot, "index.js");
const planningPath = join(packageRoot, "planning.js");
const repositoryRoot = join(packageRoot, "..", "..");

const RUST_NAPI_BINDING_FILES = [
  "crates/no-mistakes/src/napi_api.rs",
  "crates/no-mistakes/src/napi_api/codebase_bindings.rs",
  "crates/no-mistakes/src/napi_api/planning_bindings.rs",
  "crates/no-mistakes/src/napi_api/wrappers_query.rs",
  "crates/no-mistakes/src/napi_api/ci_bindings.rs",
  "crates/no-mistakes/src/napi_api/queries.rs",
  "crates/no-mistakes/src/napi_api/infra_swift.rs",
];

const RAW_NATIVE_EXPORTS = {
  testsComment: "testsCommentMarkdown",
  testsGraphMermaid: "testsGraphMermaid",
  version: "version",
};

function nativeExportNames() {
  const exports = new Set(["version"]);

  for (const relativePath of RUST_NAPI_BINDING_FILES) {
    const source = readFileSync(join(repositoryRoot, relativePath), "utf8");
    for (const match of source.matchAll(/json_binding!\(\s*\w+,\s*"([^"]+)"/g)) {
      exports.add(match[1]);
    }
    for (const match of source.matchAll(/napi\(js_name = "([^"]+)"\)/g)) {
      exports.add(match[1]);
    }
  }

  return [...exports].sort();
}

function declarationExportNames() {
  const declarations = ["index.d.ts", "index-ci-infra.d.ts"]
    .map((file) => readFileSync(join(packageRoot, file), "utf8"))
    .join("\n");
  return [
    ...new Set([...declarations.matchAll(/export function (\w+)\(/g)].map((match) => match[1])),
  ].sort();
}

function nativeExportNameForApi(apiName) {
  return RAW_NATIVE_EXPORTS[apiName] || `${apiName}Json`;
}

test("programmatic API proxies object options through async native addon calls", async () => {
  const previous = require.extensions[".node"];
  delete require.cache[require.resolve(indexPath)];
  delete require.cache[require.resolve(planningPath)];
  delete require.cache[addonPath];

  require.extensions[".node"] = (module, filename) => {
    assert.equal(filename, addonPath);
    module.exports = {
      dependenciesJson: async (json) => {
        const options = JSON.parse(json);
        if (options.root === "__locked__") {
          throw new Error("another no-mistakes invocation holds the lock");
        }
        return JSON.stringify({ command: "dependencies", options });
      },
      dependentsJson: async (json) =>
        JSON.stringify({ command: "dependents", options: JSON.parse(json) }),
      relatedJson: async (json) =>
        JSON.stringify({ command: "related", options: JSON.parse(json) }),
      analyzeProjectJson: async (json) =>
        JSON.stringify({ command: "analyzeProject", options: JSON.parse(json) }),
      symbolsJson: async (json) =>
        JSON.stringify({ command: "symbols", options: JSON.parse(json) }),
      importUsagesJson: async (json) =>
        JSON.stringify({ command: "importUsages", options: JSON.parse(json) }),
      importersJson: async (json) =>
        JSON.stringify({ command: "importers", options: JSON.parse(json) }),
      exportsOfJson: async (json) =>
        JSON.stringify({ command: "exportsOf", options: JSON.parse(json) }),
      deadExportsJson: async (json) =>
        JSON.stringify({ command: "deadExports", options: JSON.parse(json) }),
      callSitesJson: async (json) =>
        JSON.stringify({ command: "callSites", options: JSON.parse(json) }),
      resolveCheckJson: async (json) => {
        const options = JSON.parse(json);
        if (Array.isArray(options.files) && options.files.length === 0) {
          throw new Error("files must contain at least one path");
        }
        if (Object.hasOwn(options, "file") && Object.hasOwn(options, "files")) {
          throw new Error("exactly one of file or files is required");
        }
        return JSON.stringify({ command: "resolveCheck", options });
      },
      fetchesJson: async (json) =>
        JSON.stringify({ command: "fetches", options: JSON.parse(json) }),
      checkJson: async (json) => JSON.stringify({ command: "check", options: JSON.parse(json) }),
      validateMermaidMarkdownJson: async (json) =>
        JSON.stringify({ command: "validateMermaidMarkdown", options: JSON.parse(json) }),
      testsPlanJson: async (json) =>
        JSON.stringify({ command: "testsPlan", options: JSON.parse(json) }),
      testsTargetsJson: async (json) =>
        JSON.stringify({ command: "testsTargets", options: JSON.parse(json) }),
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
      serverContractsJson: async (json) =>
        JSON.stringify({ command: "serverContracts", options: JSON.parse(json) }),
      flowJson: async (json) => JSON.stringify({ command: "flow", options: JSON.parse(json) }),
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
      ciTopologyJson: async (json) =>
        JSON.stringify({ command: "ciTopology", options: JSON.parse(json) }),
      version: async () => "1.2.3",
    };
  };

  try {
    const api = require(indexPath);
    assert.deepEqual(
      await api.dependencies({
        files: ["a.mts"],
        timeout: 12,
        lockTimeout: null,
        failOnLock: true,
      }),
      {
        command: "dependencies",
        options: {
          files: ["a.mts"],
          timeout: 12,
          lockTimeout: null,
          failOnLock: true,
        },
      },
    );
    await assert.rejects(
      api.dependencies({ files: ["a.mts"], root: "__locked__", failOnLock: true }),
      /another no-mistakes invocation holds the lock/,
    );
    assert.equal((await api.dependents({ files: ["b.mts"] })).command, "dependents");
    assert.equal((await api.related({ files: ["c.mts"] })).command, "related");
    assert.deepEqual(
      await api.analyzeProject({
        timeout: 0,
        lockTimeout: 5,
        reports: [{ type: "dependencies", files: ["a.mts"] }],
      }),
      {
        command: "analyzeProject",
        options: {
          timeout: 0,
          lockTimeout: 5,
          reports: [{ type: "dependencies", files: ["a.mts"] }],
        },
      },
    );
    assert.deepEqual(
      await api.analyzeProject({
        reports: [{ type: "check", includeSuppressed: true }],
      }),
      {
        command: "analyzeProject",
        options: {
          reports: [{ type: "check", includeSuppressed: true }],
        },
      },
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
    assert.deepEqual(await api.importUsages({ filters: ["src/**"] }), {
      command: "importUsages",
      options: { filters: ["src/**"] },
    });
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
    assert.deepEqual((await api.resolveCheck({ files: ["a.ts", "b.ts"] })).options.files, [
      "a.ts",
      "b.ts",
    ]);
    await assert.rejects(api.resolveCheck({ files: [] }), /files must contain at least one path/);
    await assert.rejects(
      api.resolveCheck({ file: "a.ts", files: ["b.ts"] }),
      /exactly one of file or files is required/,
    );
    assert.equal((await api.fetches({ targets: ["/users"] })).command, "fetches");
    assert.equal((await api.check({ tsconfig: "tsconfig.json" })).command, "check");
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
    assert.equal((await api.swiftImporters({ file: "Sources/A.swift" })).command, "swiftImporters");
    assert.equal(
      (await api.swiftTestTargets({ file: "Sources/A.swift" })).command,
      "swiftTestTargets",
    );
    assert.equal((await api.ciTopology({ workflows: ["ci.yml"] })).options.workflows[0], "ci.yml");
    assert.equal(await api.version(), "1.2.3");
  } finally {
    delete require.cache[require.resolve(indexPath)];
    delete require.cache[require.resolve(planningPath)];
    delete require.cache[addonPath];
    if (previous) {
      require.extensions[".node"] = previous;
    } else {
      delete require.extensions[".node"];
    }
  }
});

test("native exports, JavaScript exports, and declarations stay in parity", async () => {
  const previous = require.extensions[".node"];
  const nativeExports = nativeExportNames();
  const observedExports = new Set();
  delete require.cache[require.resolve(indexPath)];
  delete require.cache[require.resolve(planningPath)];
  delete require.cache[addonPath];

  require.extensions[".node"] = (module, filename) => {
    assert.equal(filename, addonPath);
    module.exports = Object.fromEntries(
      nativeExports.map((name) => [
        name,
        async (json) => {
          observedExports.add(name);
          if (Object.values(RAW_NATIVE_EXPORTS).includes(name)) return name;
          return JSON.stringify({ name, options: JSON.parse(json) });
        },
      ]),
    );
  };

  try {
    const api = require(indexPath);
    const declaredExports = declarationExportNames();
    assert.deepEqual(Object.keys(api).sort(), declaredExports);

    // This export is intentionally pure JS; every other declared function
    // must cross the N-API boundary and keep returning a promise.
    for (const name of declaredExports) {
      if (name === "createWorkflowTopologyIndex") continue;
      const expectedNativeExport = nativeExportNameForApi(name);
      const result = api[name]({});
      assert.equal(typeof result.then, "function", `${name} must remain async`);
      const value = await result;
      if (Object.hasOwn(RAW_NATIVE_EXPORTS, name)) {
        assert.equal(value, expectedNativeExport);
      } else {
        assert.equal(value.name, expectedNativeExport);
      }
    }

    assert.deepEqual([...observedExports].sort(), nativeExports);
  } finally {
    delete require.cache[require.resolve(indexPath)];
    delete require.cache[require.resolve(planningPath)];
    delete require.cache[addonPath];
    if (previous) {
      require.extensions[".node"] = previous;
    } else {
      delete require.extensions[".node"];
    }
  }
});

test("native ESM imports expose every declared root API", async () => {
  const previous = require.extensions[".node"];
  delete require.cache[require.resolve(indexPath)];
  delete require.cache[require.resolve(planningPath)];
  delete require.cache[require.resolve(addonPath)];

  require.extensions[".node"] = (module, filename) => {
    assert.equal(filename, addonPath);
    module.exports = { version: async () => "1.2.3" };
  };

  try {
    const esm = await import(pathToFileURL(indexPath).href);
    const declaredExports = declarationExportNames();

    for (const name of declaredExports) {
      assert.equal(typeof esm[name], "function", `${name} must support native ESM named imports`);
    }

    assert.equal(typeof esm.ciTopology, "function");
    assert.equal(typeof esm.testsPlan, "function");
    assert.equal(typeof esm.createWorkflowTopologyIndex, "function");
    assert.equal(typeof esm.version, "function");
  } finally {
    delete require.cache[require.resolve(indexPath)];
    delete require.cache[require.resolve(planningPath)];
    delete require.cache[require.resolve(addonPath)];
    if (previous) {
      require.extensions[".node"] = previous;
    } else {
      delete require.extensions[".node"];
    }
  }
});

test("analyzeProject declarations mirror report-specific runtime requirements", () => {
  const traversalDeclarations = readFileSync(join(packageRoot, "traversal-types.d.ts"), "utf8");
  const analyzeProjectDeclarations = readFileSync(
    join(packageRoot, "analyze-project-types.d.ts"),
    "utf8",
  );
  assert.match(
    traversalDeclarations,
    /export type SymbolsSignatureImpactOptions = SymbolsOptions & \{/,
  );
  assert.match(
    readFileSync(join(packageRoot, "index.d.ts"), "utf8"),
    /options: WithInvocationOptions<SymbolsOptions>,\n\): Promise<SymbolsResult \| SignatureImpactResult>;/,
  );
  assert.match(traversalDeclarations, /mode: "signature-impact";\n  symbol: string;/);
  assert.match(
    analyzeProjectDeclarations,
    /type: "symbols"; id\?: string } & \(SymbolsListOptions \| SymbolsSignatureImpactOptions\)/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "importUsages"; id\?: string } & Omit<ImportUsagesOptions, "root">/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type BatchedTraverseOptions = TraverseOptions & Pick<ProjectOptions, "config">/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "dependencies" \| "dependents" \| "related"; id\?: string } & BatchedTraverseOptions/,
  );
  assert.doesNotMatch(
    analyzeProjectDeclarations,
    /Omit<\s*TraverseOptions,\s*"root" \| "tsconfig"/,
  );
  assert.match(
    readFileSync(join(packageRoot, "index.d.ts"), "utf8"),
    /options\?: WithInvocationOptions<ImportUsagesOptions>,\n\): Promise<ImportUsagesResult>;/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type BatchedQueueRelatedOptions = BatchedProjectOptions & \{ files: string\[\] \}/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type BatchedServerRouteRelatedOptions = BatchedProjectOptions &\n  \(\{ files: string\[\] \} \| \{ roots: string\[\] \}\)/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "playwrightRelated"; id\?: string } & Omit<PlaywrightRelatedOptions,/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type BatchedReactUsagesOptions = Pick<[\s\S]*?"root" \| "tsconfig" \| "config" \| "targets" \| "include"[\s\S]*?Required<Pick<ProjectOptions, "target">>/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type: "reactUsages"; id\?: string } & BatchedReactUsagesOptions/,
  );
  assert.match(
    analyzeProjectDeclarations,
    /type BatchedCheckOptions = Pick<[\s\S]*?"root" \| "tsconfig" \| "config" \| "include" \| "includeSuppressed"[\s\S]*?>/,
  );
  assert.match(analyzeProjectDeclarations, /type: "check"; id\?: string } & BatchedCheckOptions/);
  assert.match(
    readFileSync(join(packageRoot, "report-types.d.ts"), "utf8"),
    /domain:[\s\S]*\| "advisories";/,
  );
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export \* from "\.\/analyze-project-types";/,
  );
});

test("resolveCheck declarations mirror its mutually exclusive runtime inputs", () => {
  const declarations = readFileSync(join(packageRoot, "query-types.d.ts"), "utf8");

  assert.match(declarations, /ResolveCheckOptions = QueryFileOptions & \{\n  files\?: never;/);
  assert.match(declarations, /files: \[string, \.\.\.string\[\]\];/);
  assert.match(declarations, /file\?: never;/);
});

test("test plan declarations require current results but accept saved legacy plan documents", () => {
  const declarations = readFileSync(join(packageRoot, "test-types.d.ts"), "utf8");

  assert.match(
    declarations,
    /export interface TestPlan \{\n  \/\*\* Complete deterministic changed-file inventory/,
  );
  assert.match(declarations, /\n  changed_files: string\[\];/);
  assert.match(
    declarations,
    /export type SavedTestPlan = Omit<TestPlan, "changed_files"> & \{\n  changed_files\?: string\[\];\n\};/,
  );
  assert.match(declarations, /planJson\?: SavedTestPlan \| string;/);
  assert.match(declarations, /export type TestsPlanOptions =/);
  assert.match(declarations, /directTestOwner: true;[\s\S]*framework: TestPlanFramework;/);
  assert.match(declarations, /framework: TestPlanFramework;[\s\S]*entrypoints\?: never;/);
  assert.match(declarations, /limitPercent\?: never;[\s\S]*limitFiles\?: never;/);
  assert.match(declarations, /globalConfigFallback\?: never;[\s\S]*directTestOwner\?: false;/);
});

test("graph declarations expose GitHub Actions workflow relationships and virtual nodes", () => {
  const traversalDeclarations = readFileSync(join(packageRoot, "traversal-types.d.ts"), "utf8");
  const flowDeclarations = readFileSync(join(packageRoot, "flow-types.d.ts"), "utf8");

  for (const relationship of [
    "workflow",
    "workflow-job",
    "workflow-step",
    "workflow-needs",
    "workflow-uses",
    "workflow-run",
    "workflow-artifact",
  ]) {
    assert.match(traversalDeclarations, new RegExp(`\\| "${relationship}"`));
  }
  assert.match(traversalDeclarations, /workflowFile\?: string;/);
  assert.match(traversalDeclarations, /job\?: string;/);
  assert.match(traversalDeclarations, /step\?: number;/);
  assert.match(flowDeclarations, /"workflow-job" \| "workflow-step"/);
  assert.match(flowDeclarations, /workflowFile\?: string;/);
  assert.match(flowDeclarations, /step\?: number;/);
});

test("declarations expose invocation controls on every analysis", () => {
  const indexDeclarations = readFileSync(join(packageRoot, "index.d.ts"), "utf8");
  const invocationDeclarations = readFileSync(join(packageRoot, "invocation-types.d.ts"), "utf8");

  assert.match(invocationDeclarations, /timeout\?: number \| null;/);
  assert.match(invocationDeclarations, /lockTimeout\?: number \| null;/);
  assert.match(invocationDeclarations, /failOnLock\?: boolean;/);
  assert.match(
    indexDeclarations,
    /analyzeProject\(\n  options: WithInvocationOptions<AnalyzeProjectOptions>/,
  );

  const declarations = indexDeclarations.matchAll(
    /export function (\w+)\([\s\S]*?\): Promise<[^;]+>;/g,
  );
  for (const [declaration, name] of declarations) {
    if (name === "version") {
      assert.doesNotMatch(declaration, /WithInvocationOptions/);
    } else {
      assert.match(declaration, /WithInvocationOptions/, `${name} must accept invocation options`);
    }
  }
});
