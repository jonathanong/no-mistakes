const { assert, test, readFileSync, join, packageRoot, repositoryRoot } = require("./context");
const { createRequire } = require("node:module");

test("single-enqueue option types compile from the public package root", () => {
  const ts = createRequire(require.resolve("@storybook/react"))("typescript");
  const program = ts.createProgram(
    [join(repositoryRoot, "fixtures/node-api/no-single-enqueue-in-loop-types/options.ts")],
    {
      strict: true,
      skipLibCheck: true,
      noEmit: true,
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.ESNext,
      moduleResolution: ts.ModuleResolutionKind.Bundler,
      baseUrl: repositoryRoot,
      paths: { "no-mistakes": ["packages/no-mistakes/index.d.ts"] },
      ignoreDeprecations: "6.0",
      types: [],
    },
  );
  assert.deepEqual(
    ts
      .getPreEmitDiagnostics(program)
      .map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n")),
    [],
  );
});

test("queue wait kinds compile when imported from the package root", { timeout: 20_000 }, () => {
  const { createRequire } = require("node:module");
  const dependencyRequire = createRequire(require.resolve("@storybook/react"));
  const ts = dependencyRequire("typescript");
  const { repositoryRoot } = require("./context");
  const program = ts.createProgram(
    [join(repositoryRoot, "fixtures", "node-api", "queue-wait-kinds", "public.mts")],
    {
      noEmit: true,
      strict: true,
      skipLibCheck: true,
      types: [],
      target: ts.ScriptTarget.ESNext,
      module: ts.ModuleKind.NodeNext,
      moduleResolution: ts.ModuleResolutionKind.NodeNext,
      baseUrl: repositoryRoot,
      paths: { "no-mistakes": ["packages/no-mistakes/index.d.ts"] },
      ignoreDeprecations: "6.0",
    },
  );
  assert.deepEqual(
    ts
      .getPreEmitDiagnostics(program)
      .map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n")),
    [],
  );
  assert.match(
    readFileSync(join(packageRoot, "index.d.ts"), "utf8"),
    /export type \{ QueueWaitFindingKind \} from "\.\/queue-report-types";/,
  );
});
