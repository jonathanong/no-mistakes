const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const ts = require("typescript");

test("changes report and planner public contracts compile", () => {
  const program = ts.createProgram(
    [join(__dirname, "../../../fixtures/node-api/changes-types/public.mts")],
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
      .map((entry) => ts.flattenDiagnosticMessageText(entry.messageText, "\n")),
    [],
  );
});
