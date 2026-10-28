const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("live repository scan options expose roots, fixtures, enumerators, and unresolved policy", () => {
  const types = readFileSync(join(packageRoot, "test-no-live-repository-scan-types.d.ts"), "utf8");
  assert.match(types, /export interface TestNoLiveRepositoryScanOptions/);
  assert.match(types, /export type TestNoLiveRepositoryScanRoot/);
  assert.match(types, /export interface TestNoLiveRepositoryScanEnumerator/);
  assert.match(types, /roots\?: TestNoLiveRepositoryScanRoot\[\];/);
  assert.match(types, /excludeRoots\?: string \| string\[\];/);
  assert.match(types, /fixtureDirs\?: string\[\];/);
  assert.match(types, /enumerators\?: TestNoLiveRepositoryScanEnumerator\[\];/);
  assert.match(types, /unresolvedRoot\?: "advise" \| "report";/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/test-no-live-repository-scan-types";/,
  );
});
