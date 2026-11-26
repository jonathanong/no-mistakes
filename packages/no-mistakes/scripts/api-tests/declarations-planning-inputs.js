const { createRequire } = require("node:module");
const { assert, join, packageRoot, readFileSync, repositoryRoot, test } = require("./context");
const ts = createRequire(require.resolve("@storybook/react"))("typescript");

// no-mistakes-timeout-reason: typechecks the public planning input fixture through the TypeScript program API
test("planning inputs enforce Git revision pairs and expose both patch forms", () => {
  const fixture = join(repositoryRoot, "fixtures/node-api/planning-parity/public.mts");
  const program = ts.createProgram([fixture], {
    noEmit: true,
    strict: true,
    skipLibCheck: true,
    types: [],
    target: ts.ScriptTarget.ESNext,
    module: ts.ModuleKind.NodeNext,
    moduleResolution: ts.ModuleResolutionKind.NodeNext,
  });
  assert.deepEqual(
    ts
      .getPreEmitDiagnostics(program)
      .map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n")),
    [],
  );
}, 20_000);

test("test plan declarations require current results but accept saved legacy plan documents", () => {
  const declarations = ["test-types.d.ts", "test-plan-document-types.d.ts"]
    .map((file) => readFileSync(join(packageRoot, file), "utf8"))
    .join("\n");

  for (const pattern of [
    /export interface TestPlan \{\n  \/\*\* Complete deterministic changed-file inventory/,
    /\n  changedFiles: string\[\];/,
    /export type SavedTestPlan = Omit<TestPlan, "changedFiles"> & \{\s*changedFiles\?: string\[\];/,
    /planJson\?: SavedTestPlan \| string;/,
    /export interface TestsWhyOptions \{[\s\S]*plan\?: string;\n  planJson\?: SavedTestPlan \| string;/,
    /export interface TestsPlanOptionsBase \{/,
    /export type TestsPlanOptions =/,
    /export type TestPlanFramework =\n  \| "vitest"\n  \| "playwright"\n  \| "dotnet"\n  \| "swift"\n  \| "python"\n  \| "go"\n  \| "cargo"\n  \| "rails"\n  \| "php"\n  \| "java"\n  \| "kotlin"\n  \| "elixir"\n  \| "dart"\n  \| "jest";/,
    /directTestOwner: true;[\s\S]*framework: TestPlanFramework;/,
    /framework: TestPlanFramework;[\s\S]*entrypoints\?: never;/,
    /limitPercent\?: never;[\s\S]*limitFiles\?: never;/,
    /globalConfigFallback\?: never;[\s\S]*directTestOwner\?: false(?:;|\s*\})/,
    /export interface TestPlanIncompleteness \{/,
    /export type TestPlanIncompletenessKind = "dependency-baseline-missing";/,
    /incomplete\?: TestPlanIncompleteness\[\];/,
  ]) {
    assert.match(declarations, pattern);
  }
  assert.doesNotMatch(declarations, /TestPlan\["incomplete"\]/);
});
