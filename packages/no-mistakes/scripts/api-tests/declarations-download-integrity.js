const { assert, test, readFileSync, join, packageRoot, repositoryRoot } = require("./context");
const { createRequire } = require("node:module");

test("download integrity exports scoped shell, sequencing and annotation options", () => {
  const declaration = readFileSync(
    join(packageRoot, "github-actions-download-integrity-types.d.ts"),
    "utf8",
  );
  assert.match(declaration, /export interface GithubActionsDownloadIntegrityOptions/);
  assert.match(declaration, /trustedHosts\?: string\[\];/);
  assert.match(declaration, /downloadHelpers\?: string\[\];/);
  assert.match(declaration, /shellSourceInclude\?: string\[\];/);
  assert.match(declaration, /strictSequencing\?: boolean;/);
  assert.match(
    declaration,
    /export type GithubActionsDownloadIntegritySkipAnnotationScope = "step" \| "line";/,
  );
  assert.match(
    declaration,
    /skipAnnotationScope\?: GithubActionsDownloadIntegritySkipAnnotationScope;/,
  );
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      "./github-actions-download-integrity-types",
    ),
  );
});

test("download integrity annotation scope is a named public union", () => {
  const dependencyRequire = createRequire(require.resolve("@storybook/react"));
  const ts = dependencyRequire("typescript");
  const program = ts.createProgram(
    [join(repositoryRoot, "fixtures/node-api/download-integrity-types/options.ts")],
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
