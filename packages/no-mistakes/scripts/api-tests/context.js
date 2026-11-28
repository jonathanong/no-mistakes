const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { existsSync, readFileSync, writeFileSync, mkdtempSync } = require("node:fs");
const { createRequire } = require("node:module");
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
  "crates/no-mistakes-napi/src/lib.rs",
  "crates/no-mistakes-napi/src/diagnostics.rs",
  "crates/no-mistakes-napi/src/postgres_source.rs",
  "crates/no-mistakes-napi/src/postgres_bindings.rs",
  "crates/no-mistakes-napi/src/typescript_modules.rs",
  "crates/no-mistakes-napi/src/codebase_bindings.rs",
  "crates/no-mistakes-napi/src/planning_bindings.rs",
  "crates/no-mistakes-napi/src/wrappers_query.rs",
  "crates/no-mistakes-napi/src/ci_bindings.rs",
  "crates/no-mistakes-napi/src/queries.rs",
  "crates/no-mistakes-napi/src/infra_swift.rs",
];

const RAW_NATIVE_EXPORTS = {
  testsComment: "testsCommentMarkdown",
  testsGraphMermaid: "testsGraphMermaid",
  version: "version",
};
const PURE_JAVASCRIPT_EXPORTS = new Set([
  "collectDiagnostics",
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
    "index-postgres-lock-inventory.d.ts",
    "index-queues.d.ts",
    "invocation-types.d.ts",
    "diagnostics-collector.d.ts",
    "executed-postgres-sql-types.d.ts",
    "changes-types.d.ts",
  ]
    .map((file) => readFileSync(join(packageRoot, file), "utf8"))
    .join("\n");
  return [
    ...new Set(
      [...declarations.matchAll(/export function (\w+)(?:<[^;{]*?>)?\(/g)].map((match) => match[1]),
    ),
  ].sort();
}

function nativeExportNameForApi(apiName) {
  return RAW_NATIVE_EXPORTS[apiName] || `${apiName}Json`;
}

let cachedTs;
const cachedSourceFiles = new Map();

function loadTypeScript() {
  if (!cachedTs) {
    // Resolve the compiler through an existing direct dependency, without adding a test dependency.
    cachedTs = createRequire(require.resolve("@storybook/react"))("typescript");
  }
  return cachedTs;
}

function assertFixturesCompile(rootNames, compilerOptions) {
  const ts = loadTypeScript();
  const options = typeof compilerOptions === "function" ? compilerOptions(ts) : compilerOptions;
  const host = ts.createCompilerHost(options);
  const originalGetSourceFile = host.getSourceFile.bind(host);
  host.getSourceFile = (fileName, languageVersionOrOptions, ...rest) => {
    const version =
      typeof languageVersionOrOptions === "object" && languageVersionOrOptions !== null
        ? languageVersionOrOptions.languageVersion
        : languageVersionOrOptions;
    const cacheKey = `${version}:${fileName}`;
    if (cachedSourceFiles.has(cacheKey)) return cachedSourceFiles.get(cacheKey);
    const sourceFile = originalGetSourceFile(fileName, languageVersionOrOptions, ...rest);
    if (sourceFile) cachedSourceFiles.set(cacheKey, sourceFile);
    return sourceFile;
  };
  const program = ts.createProgram(rootNames, options, host);
  assert.deepEqual(
    ts
      .getPreEmitDiagnostics(program)
      .map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n")),
    [],
  );
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
  assertFixturesCompile,
};
