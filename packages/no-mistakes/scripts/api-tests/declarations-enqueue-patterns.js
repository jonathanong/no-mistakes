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

function assertPublicNodeNextFixtureCompiles(...fixtureParts) {
  const dependencyRequire = createRequire(require.resolve("@storybook/react"));
  const ts = dependencyRequire("typescript");
  const program = ts.createProgram([join(repositoryRoot, ...fixtureParts)], {
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
  });
  assert.deepEqual(
    ts
      .getPreEmitDiagnostics(program)
      .map((diagnostic) => ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n")),
    [],
  );
}

test("queue wait kinds compile when imported from the package root", { timeout: 20_000 }, () => {
  assertPublicNodeNextFixtureCompiles("fixtures", "node-api", "queue-wait-kinds", "public.mts");
  assert.match(
    readFileSync(join(packageRoot, "index.d.ts"), "utf8"),
    /export type \{ QueueRateLimitFindingKind, QueueWaitFindingKind \} from "\.\/queue-report-types";/,
  );
});

test(
  "queue rate-limit finding kinds are public and compile from the package root",
  { timeout: 20_000 },
  () => {
    const declarations = readFileSync(join(packageRoot, "queue-report-types.d.ts"), "utf8");
    assert.match(
      declarations,
      /export type QueueRateLimitFindingKind = "shared-queue-rate-limit" \| "worker-rate-limit-per-process";/,
    );
    assert.match(
      declarations,
      /export interface QueueCheckOptions extends ProjectOptions \{[^}]*config\?: string;/,
    );
    assert.match(declarations, /export interface QueueCheckReport \{/);
    assertPublicNodeNextFixtureCompiles(
      "fixtures",
      "node-api",
      "queue-rate-limit-kinds",
      "public.mts",
    );
  },
);
