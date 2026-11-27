const {
  assert,
  test,
  readFileSync,
  join,
  packageRoot,
  assertFixturesCompile,
} = require("./context");

const declarations =
  readFileSync(join(packageRoot, "diagnostics-types.d.ts"), "utf8") +
  readFileSync(join(packageRoot, "diagnostics-collector.d.ts"), "utf8");

test("verbose diagnostics exposes named report contracts and preserves operation argument tuples", () => {
  for (const name of [
    "DiagnosticsResult<T>",
    "VerboseDiagnostics",
    "DiagnosticTiming",
    "PhaseResourceDiagnostic",
    "HotFileDiagnostic",
    "DiagnosticOperations",
  ]) {
    assert.ok(declarations.includes(`export interface ${name} {`), name);
  }
  assert.match(declarations, /export interface DiagnosticsError extends Error/);
  assert.match(declarations, /export type DiagnosticFilePass =/);
  for (const pass of [
    "parse",
    "value_flow.extract",
    "value_flow.finalize",
    "postgres.annotation",
  ]) {
    assert.ok(declarations.includes(`"${pass}"`));
  }
  for (const field of [
    "elapsedMs",
    "durationMs",
    "wallMs",
    "userMs",
    "systemMs",
    "utilization",
    "peakRssDeltaKiB",
    "units",
  ]) {
    assert.match(declarations, new RegExp(`${field}: number;`));
  }
  assert.match(declarations, /timings: DiagnosticTiming\[\];/);
  assert.match(declarations, /phases: PhaseResourceDiagnostic\[\];/);
  assert.match(declarations, /hotFiles: HotFileDiagnostic\[\];/);
  assert.match(declarations, /work: Record<string, number>;/);
  assert.match(declarations, /peakRssKiB\?: number;/);
  assert.match(declarations, /diagnostics\?: VerboseDiagnostics;/);
  assert.match(
    declarations,
    /export type DiagnosticArguments<K extends keyof DiagnosticOperations>/,
  );
  assert.match(declarations, /collectDiagnostics<K extends keyof DiagnosticOperations>\(/);
  assert.match(declarations, /\.\.\.args: DiagnosticArguments<K>/);
  assert.match(
    declarations,
    /Promise<DiagnosticsResult<Awaited<ReturnType<DiagnosticOperations\[K\]>>>>/,
  );
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export \* from "\.\/diagnostics-types";/,
  );
});

test("diagnostics selectors cover public analyses and text renderers", () => {
  const root = readFileSync(join(packageRoot, "index.js"), "utf8");
  const excluded = new Set([
    "collectDiagnostics",
    "createWorkflowTopologyIndex",
    "diagnosticsSnapshot",
    "version",
    "writePlanningImpactArtifacts",
  ]);
  const expected = [...root.matchAll(/module\.exports\.(\w+) =/g)]
    .map((match) => match[1])
    .filter((name) => !excluded.has(name))
    .sort();
  const actual = [...declarations.matchAll(/  (\w+): typeof Api\.(\w+);/g)]
    .map((match) => {
      assert.equal(match[1], match[2]);
      return match[1];
    })
    .sort();
  assert.deepEqual(actual, expected);
});

// no-mistakes-timeout-reason: typechecks the public diagnostics fixture through the TypeScript program API
test("diagnostics collector overloads retain required arguments and correlated result types", () => {
  const fixture = join(
    packageRoot,
    "..",
    "..",
    "fixtures",
    "node-api",
    "diagnostics-types",
    "public.mts",
  );
  assertFixturesCompile([fixture], (ts) => ({
    noEmit: true,
    strict: true,
    skipLibCheck: true,
    types: [],
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
  }));
}, 20_000);
