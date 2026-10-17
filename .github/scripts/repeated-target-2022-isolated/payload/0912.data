const assert = require("node:assert/strict");
const { createRequire } = require("node:module");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { runInNewContext } = require("node:vm");
const test = globalThis.test || require("node:test").test;
const root = join(__dirname, "..", "..", "..");

test("audit types require exclusive plan and observations inputs", () => {
  const parser = require.resolve("@typescript-eslint/parser", {
    paths: [join(root, "packages", "eslint-plugin-no-mistakes")],
  });
  const ts = createRequire(parser)("typescript");
  const program = ts.createProgram(
    [join(root, "test-cases", "tests-audit", "options-consumer.ts")],
    {
      noEmit: true,
      strict: true,
      skipLibCheck: true,
      module: ts.ModuleKind.Node16,
      moduleResolution: ts.ModuleResolutionKind.Node16,
      target: ts.ScriptTarget.ES2022,
    },
  );
  const errors = ts.getPreEmitDiagnostics(program);
  assert.deepEqual(
    errors.map((error) => ts.flattenDiagnosticMessageText(error.messageText, "\n")),
    [],
  );
});

test("documented Istanbul adapter counts statement, function and branch execution", () => {
  const docs = readFileSync(join(root, "docs", "cli", "tests-audit.md"), "utf8");
  const match = docs.match(/\.filter\(file => ([\s\S]+?)\)\n\s+\.map/);
  assert.ok(match, "producer execution filter must remain executable");
  // Execute the actual recipe expression, including function-only and branch-only traces.
  const executed = runInNewContext(`file => ${match[1]}`);
  const cases = JSON.parse(
    readFileSync(join(root, "test-cases", "tests-audit", "istanbul-counter-cases.json"), "utf8"),
  );
  for (const row of cases) assert.equal(executed(row), row.executed);
});

test("documented adapter uses runner evidence for test-file execution", () => {
  const docs = readFileSync(join(root, "docs", "cli", "tests-audit.md"), "utf8");
  const match = docs.match(
    /if \(row\.testFileExecuted === true\) executedFiles\.push\(row\.testFile\);/,
  );
  assert.ok(match, "producer must use explicit test-source execution evidence");
  const cases = JSON.parse(
    readFileSync(
      join(root, "test-cases", "tests-audit", "adapter-self-execution-cases.json"),
      "utf8",
    ),
  );
  for (const row of cases) {
    const executedFiles = [];
    // Run the actual recipe statement; inventory and unknown tracing are not execution proof.
    runInNewContext(match[0], { row, executedFiles });
    assert.deepEqual(executedFiles, row.testFileExecuted ? [row.testFile] : []);
  }
});
