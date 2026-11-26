const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const ts = require("typescript");

test("Playwright and server route report fields compile through the public declarations", () => {
  const program = ts.createProgram(
    [join(__dirname, "../../../fixtures/node-api/graph-reports/public.mts")],
    {
      noEmit: true,
      strict: true,
      skipLibCheck: true,
      types: [],
      target: ts.ScriptTarget.ESNext,
      module: ts.ModuleKind.NodeNext,
    },
  );
  assert.deepEqual(
    ts
      .getPreEmitDiagnostics(program)
      .map((entry) => ts.flattenDiagnosticMessageText(entry.messageText, "\n")),
    [],
  );
});
