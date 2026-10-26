require("./api-tests/proxy");
require("./api-tests/planning");
require("./api-tests/exports");
require("./api-tests/declarations-core");
require("./api-tests/declarations-unbounded");
require("./api-tests/declarations-export-invariants");
require("./api-tests/declarations-mutable");
require("./api-tests/declarations-test-state-mutation");
require("./api-tests/declarations-postgres");
require("./api-tests/declarations-graph");
require("./api-tests/declarations-executor-diagnostics");
require("./api-tests/declarations-analysis");
require("./api-tests/declarations-sql");
require("./api-tests/declarations-audit");
require("./api-tests/declarations-runtime");

const { assert, test, readFileSync, join, packageRoot } = require("./api-tests/context");
test("annotation literal helpers remain an exported option and verdict profile", () => {
  for (const file of ["postgres-query-annotation-types.d.ts", "check-report-types.d.ts"]) {
    assert.match(readFileSync(join(packageRoot, file), "utf8"), /sqlLiteralHelpers\?: string\[\];/);
  }
});

test("Playwright coverage exposes named optional matcher warnings", () => {
  const declarations = readFileSync(join(packageRoot, "report-types.d.ts"), "utf8");
  assert.match(declarations, /export interface PlaywrightCoverageReport/);
  assert.match(declarations, /warnings\?: string\[\]/);
  const index = readFileSync(join(packageRoot, "index.d.ts"), "utf8");
  assert.match(index, /playwrightCheck\([\s\S]*?Promise<PlaywrightCoverageReport>/);
});

test("coverage assertion owner options are public and bounded", () => {
  const declaration = readFileSync(
    join(packageRoot, "coverage-assertion-owner-types.d.ts"),
    "utf8",
  );
  for (const name of [
    "CoverageProducerFunction",
    "CoverageAssertionImport",
    "CoverageAssertionOwnership",
    "CoverageProducerAssertionOwnerOptions",
  ]) {
    assert.match(declaration, new RegExp("export interface " + name));
  }
  for (const field of [
    "producers:",
    "assertions:",
    "ownership?:",
    "maxDepth?:",
    "maxNodes?:",
    "unanalyzable?:",
  ])
    assert.ok(declaration.includes(field));
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      "./coverage-assertion-owner-types",
    ),
  );
});
