const { createRequire } = require("node:module");
const { assert, join, repositoryRoot, test } = require("./context");
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
