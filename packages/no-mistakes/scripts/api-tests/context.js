const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { existsSync, readFileSync, writeFileSync, mkdtempSync } = require("node:fs");
const { join, resolve } = require("node:path");
const { tmpdir } = require("node:os");
const { pathToFileURL } = require("node:url");

const packageRoot = join(__dirname, "..", "..");
const indexPath = join(packageRoot, "index.js");
const planningPath = join(packageRoot, "planning.js");
const repositoryRoot = join(packageRoot, "..", "..");
const addonPath = join(repositoryRoot, "fixtures", "napi", "test-addon.js");
const planningFixturePath = join(
  repositoryRoot,
  "fixtures",
  "node-api",
  "planning-temp-cleanup",
  "plan.json",
);

const RUST_NAPI_BINDING_FILES = [
  "crates/no-mistakes/src/napi_api.rs",
  "crates/no-mistakes/src/napi_api/diagnostics.rs",
  "crates/no-mistakes/src/napi_api/postgres_source.rs",
  "crates/no-mistakes/src/napi_api/typescript_modules.rs",
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
const PURE_JAVASCRIPT_EXPORTS = new Set([
  "createWorkflowTopologyIndex",
  "writePlanningImpactArtifacts",
]);

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
  const declarations = [
    "index.d.ts",
    "index-ci-infra.d.ts",
    "invocation-types.d.ts",
    "executed-postgres-sql-types.d.ts",
  ]
    .map((file) => readFileSync(join(packageRoot, file), "utf8"))
    .join("\n");
  return [
    ...new Set([...declarations.matchAll(/export function (\w+)\(/g)].map((match) => match[1])),
  ].sort();
}

function nativeExportNameForApi(apiName) {
  return RAW_NATIVE_EXPORTS[apiName] || `${apiName}Json`;
}

module.exports = {
  assert,
  test,
  existsSync,
  readFileSync,
  writeFileSync,
  mkdtempSync,
  join,
  resolve,
  tmpdir,
  pathToFileURL,
  packageRoot,
  indexPath,
  planningPath,
  repositoryRoot,
  addonPath,
  planningFixturePath,
  RUST_NAPI_BINDING_FILES,
  RAW_NATIVE_EXPORTS,
  PURE_JAVASCRIPT_EXPORTS,
  nativeExportNames,
  declarationExportNames,
  nativeExportNameForApi,
};
