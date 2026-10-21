const { assert, test, readFileSync, join, packageRoot, repositoryRoot } = require("./context");

test(
  "timeout policy public type contracts compile without losing option fields",
  { timeout: 20_000 },
  () => {
    const { createRequire } = require("node:module");
    // Resolve the compiler through an existing direct dependency, without adding a test dependency.
    const dependencyRequire = createRequire(require.resolve("@storybook/react"));
    const ts = dependencyRequire("typescript");
    const program = ts.createProgram(
      [join(repositoryRoot, "fixtures", "test-timeout-policy", "type-contracts", "public.mts")],
      {
        noEmit: true,
        strict: true,
        skipLibCheck: true,
        types: [],
        target: ts.ScriptTarget.ESNext,
        module: ts.ModuleKind.NodeNext,
        moduleResolution: ts.ModuleResolutionKind.NodeNext,
        baseUrl: repositoryRoot,
        paths: {
          "no-mistakes/test-timeout-policy": [
            "packages/no-mistakes/test-timeout-policy/index.d.mts",
          ],
          "no-mistakes/test-timeout-policy/plugin": [
            "packages/no-mistakes/test-timeout-policy/plugin.d.mts",
          ],
        },
        ignoreDeprecations: "6.0",
      },
    );
    const diagnostics = ts.getPreEmitDiagnostics(program);
    assert.deepEqual(
      diagnostics.map((diagnostic) =>
        ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n"),
      ),
      [],
    );
  },
);

test("active progress exposes named public snapshot contracts and a live-only polling API", () => {
  const declarations = readFileSync(join(packageRoot, "invocation-types.d.ts"), "utf8");
  assert.match(declarations, /diagnosticsId\?: string/);
  for (const name of ["ActiveTimingSpan", "CompletedTimingSpan", "DiagnosticsProgressSnapshot"]) {
    assert.match(declarations, new RegExp(`export interface ${name}`));
  }
  for (const field of ["startedMs", "elapsedMs", "startedSpans", "completedSpans"]) {
    assert.match(declarations, new RegExp(`${field}: number`));
  }
  assert.match(declarations, /active: ActiveTimingSpan\[\]/);
  assert.match(declarations, /completed: CompletedTimingSpan\[\]/);
  const index = readFileSync(join(packageRoot, "index.d.ts"), "utf8");
  assert.match(index, /export \* from "\.\/types"/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export \* from "\.\/invocation-types"/,
  );
  assert.match(declarations, /export function diagnosticsSnapshot\(/);
  assert.match(declarations, /DiagnosticsProgressSnapshot \| null/);
});

test("Cargo warning declarations retain exported structured warning contracts", () => {
  const declarations = readFileSync(join(packageRoot, "test-types.d.ts"), "utf8");
  assert.match(declarations, /export interface TestPlanWarning \{/);
  assert.match(declarations, /export interface TestTargetWarning \{/);
  assert.match(declarations, /cargo-target-unresolved/);
});

test("effects alternatives expose a named optional public evidence type", () => {
  const named = readFileSync(join(packageRoot, "named-query-types.d.ts"), "utf8");
  assert.match(
    named,
    /export interface EffectCallAlternative \{\s+callee: string;\s+category\?: string;/,
  );
  assert.match(named, /alternatives\?: EffectCallAlternative\[\];/);
  assert.match(named, /export interface EffectCallSite/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export \* from "\.\/named-query-types";/,
  );
  assert.match(
    readFileSync(join(packageRoot, "index.d.ts"), "utf8"),
    /export \* from "\.\/types";/,
  );
  assert.match(
    readFileSync(join(packageRoot, "index-ci-infra.d.ts"), "utf8"),
    /effects\(\s*options: WithInvocationOptions<EffectsOptions>,?\s*\): Promise<WithInvocationResult<EffectsReport>>/,
  );
});

test("forbidden-call predicates export bounded literal and unknown-policy declarations", () => {
  const declarations = readFileSync(join(packageRoot, "forbidden-call-types.d.ts"), "utf8");
  for (const name of [
    "ForbiddenCallArgumentLiteral",
    "ForbiddenCallUnknownArguments",
    "ForbiddenCallArgumentPredicate",
  ]) {
    assert.match(declarations, new RegExp(`export type ${name} =`));
  }
  assert.match(declarations, /export interface ForbiddenCallArgumentConditions/);
  assert.match(declarations, /"finding" \| "ignore"/);
  assert.match(declarations, /arrayIncludes: ForbiddenCallArgumentLiteral/);
  assert.match(declarations, /arrayExcludes: ForbiddenCallArgumentLiteral/);
  assert.match(declarations, /property\?: string/);
  assert.match(readFileSync(join(packageRoot, "index.d.ts"), "utf8"), /export \* from "\.\/types"/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/forbidden-call-types"/,
  );
});

test("analysis object result metadata compiles while pure and array APIs stay unchanged", () => {
  const { createRequire } = require("node:module");
  const dependencyRequire = createRequire(require.resolve("@storybook/react"));
  const ts = dependencyRequire("typescript");
  const program = ts.createProgram(
    [join(repositoryRoot, "fixtures/node-api/jobs-pool-result-types/public.mts")],
    {
      noEmit: true,
      strict: true,
      skipLibCheck: true,
      types: [],
      target: ts.ScriptTarget.ESNext,
      module: ts.ModuleKind.NodeNext,
      moduleResolution: ts.ModuleResolutionKind.NodeNext,
    },
  );
  assert.deepEqual(
    ts
      .getPreEmitDiagnostics(program)
      .map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n")),
    [],
  );
  const declarations = readFileSync(join(packageRoot, "invocation-types.d.ts"), "utf8");
  assert.match(declarations, /export interface InvocationResultMetadata/);
  assert.match(declarations, /export type WithInvocationResult<T>/);
});
