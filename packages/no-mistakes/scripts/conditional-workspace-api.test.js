const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { resolve } = require("node:path");
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const root = resolve(
  __dirname,
  "../../../test-cases/codebase-analysis/conditional-workspace-imports/fixture",
);
const targets = [
  "packages/app/internal/target.mts",
  "packages/app/relative.mts",
  "packages/lib/target-a.mts",
  "packages/lib/target-b.mts",
];
const reverse = [
  "packages/app/conditional.mts",
  "packages/app/conditional.test.mts",
  "packages/app/sequential.mts",
  "packages/app/top-level.mts",
];
const files = ["conditional.mts", "conditional.test.mts", "sequential.mts", "top-level.mts"];
const relationshipSets = [
  ["import"],
  ["import-dynamic"],
  ["import", "workspace"],
  ["import-dynamic", "workspace"],
];
const paths = (report) =>
  report.files
    .map((file) => (typeof file === "string" ? file : file.path).replaceAll("\\", "/"))
    .sort();
const graphId = (file, relationships) => `graph:${file}:${relationships.join("+")}`;
// Independent graph queries share one request-scoped analysis per facade.
const reports = [
  ...files.flatMap((file) =>
    relationshipSets.map((relationships) => ({
      type: "dependencies",
      id: graphId(file, relationships),
      files: [`packages/app/${file}`],
      relationships,
    })),
  ),
  {
    type: "dependencies",
    id: "static",
    files: ["packages/app/static.mts"],
    relationships: ["import", "workspace"],
  },
  {
    type: "dependencies",
    id: "exact",
    files: ["packages/app/conditional.test.mts"],
    relationships: ["import-dynamic", "workspace"],
    depth: 10,
    projection: "paths",
  },
  {
    type: "dependents",
    id: "dependents",
    files: ["packages/lib/target-a.mts"],
    relationships: ["import-dynamic"],
  },
  {
    type: "related",
    id: "related",
    files: ["packages/lib/target-a.mts"],
    relationships: ["import-dynamic"],
  },
  {
    type: "dependencies",
    id: "type-dynamic",
    files: ["packages/app/type-only.mts"],
    relationships: ["import-dynamic"],
  },
  {
    type: "dependencies",
    id: "type-workspace",
    files: ["packages/app/type-only.mts"],
    relationships: ["workspace"],
  },
  {
    type: "resolveCheck",
    id: "computed-resolve",
    file: "packages/app/computed.mts",
  },
  {
    type: "dependencies",
    id: "computed-graph",
    files: ["packages/app/computed.mts"],
    relationships: ["import-dynamic", "workspace"],
  },
];
const prepared = new Map();
let analyzeProjectCalls = 0;
function preparedReports(facade) {
  if (!prepared.has(facade)) {
    const promise = (async () => {
      const api = facade === "cjs" ? require("../index.js") : await import("../index.mjs");
      analyzeProjectCalls += 1;
      const pending = api.analyzeProject({
        root,
        tsconfig: "tsconfig.json",
        reports,
      });
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.equal(report.reports.length, reports.length);
      return new Map(report.reports.map(({ id, result }) => [id, result]));
    })();
    prepared.set(facade, promise);
  }
  return prepared.get(facade);
}
async function facadeResult(id) {
  const cjsPending = preparedReports("cjs");
  assert.ok(cjsPending instanceof Promise);
  const report = (await cjsPending).get(id);
  const esmPending = preparedReports("esm");
  assert.ok(esmPending instanceof Promise);
  assert.deepEqual((await esmPending).get(id), report, id);
  return report;
}

for (const file of files) {
  for (const relationships of relationshipSets) {
    const id = graphId(file, relationships);
    test(
      `compiled CJS and ESM preserve conditional workspace imports across graph projections: ${file} ${relationships.join("+")}`,
      { skip: !compiled },
      async () => {
        const report = await facadeResult(id);
        assert.deepEqual(paths(report), targets, id);
        const library = report.files.find(
          (entry) => entry.path.replaceAll("\\", "/") === "packages/lib/target-a.mts",
        );
        assert.ok(
          library.via.includes(
            file === "top-level.mts" ? "dynamic-import" : "conditional-dynamic-import",
          ),
        );
      },
    );
  }
}

test(
  "compiled CJS and ESM preserve conditional workspace imports across graph projections: static",
  { skip: !compiled },
  async () => {
    assert.deepEqual(paths(await facadeResult("static")), targets);
  },
);

test(
  "compiled CJS and ESM preserve conditional workspace imports across graph projections: exact",
  { skip: !compiled },
  async () => {
    assert.deepEqual(paths(await facadeResult("exact")), targets);
  },
);

test(
  "compiled CJS and ESM preserve conditional workspace imports across graph projections: dependents",
  { skip: !compiled },
  async () => {
    assert.deepEqual(paths(await facadeResult("dependents")), reverse);
  },
);

test(
  "compiled CJS and ESM preserve conditional workspace imports across graph projections: related",
  { skip: !compiled },
  async () => {
    assert.deepEqual(paths(await facadeResult("related")), reverse);
  },
);

test(
  "compiled CJS and ESM preserve conditional workspace imports across graph projections: type-dynamic",
  { skip: !compiled },
  async () => {
    assert.deepEqual(paths(await facadeResult("type-dynamic")), []);
  },
);

test(
  "compiled CJS and ESM preserve conditional workspace imports across graph projections: type-workspace",
  { skip: !compiled },
  async () => {
    assert.deepEqual(paths(await facadeResult("type-workspace")), ["packages/lib/type-target.mts"]);
  },
);

test(
  "compiled CJS and ESM preserve conditional workspace imports across graph projections: computed-resolve",
  { skip: !compiled },
  async () => {
    const computed = await facadeResult("computed-resolve");
    assert.equal(computed.allResolve, false);
    assert.equal(computed.imports.length, 2);
    assert.ok(computed.imports.every((entry) => entry.computed && entry.status === "unresolved"));
  },
);

test(
  "compiled CJS and ESM preserve conditional workspace imports across graph projections: computed-graph",
  { skip: !compiled },
  async () => {
    assert.deepEqual(paths(await facadeResult("computed-graph")), []);
  },
);

test(
  "compiled CJS and ESM preserve conditional workspace imports across graph projections: prepared report parity",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const options = {
      root,
      files: ["packages/app/conditional.test.mts"],
      relationships: ["import-dynamic", "workspace"],
      tsconfig: "tsconfig.json",
      depth: 10,
      projection: "paths",
    };
    const pending = cjs.dependencies(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(paths(report), paths(await facadeResult("exact")));
    const esmPending = esm.dependencies(options);
    assert.ok(esmPending instanceof Promise);
    assert.deepEqual(await esmPending, report);
  },
);

test(
  "compiled CJS and ESM preserve conditional workspace imports across graph projections: one prepared analysis per facade",
  { skip: !compiled },
  async () => {
    await facadeResult("static");
    assert.equal(analyzeProjectCalls, 2);
  },
);
