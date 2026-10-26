const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("test state mutation options require explicit entries and expose the SQL policy", () => {
  const types = readFileSync(
    join(packageRoot, "postgres-test-state-mutation-types.d.ts"),
    "utf8",
  );
  assert.match(types, /export interface PostgresNoTestStateMutationOptions/);
  assert.match(types, /testGlobs: string\[\];/);
  assert.match(types, /helperRoots\?: string\[\];/);
  for (const field of ["executorNames", "executorFactoryNames", "executorTypeNames"])
    assert.ok(types.includes(`${field}?: string[];`));
  assert.match(types, /importSpecifier\?: string;/);
  assert.match(types, /trustedSqlTags\?: \{ module: string; name: string \}\[\];/);
  assert.match(types, /unanalyzable\?: Unanalyzable;/);
  assert.match(types, /unanalyzableSql\?: Unanalyzable \| "fail";/);
  assert.doesNotMatch(types, /sqlInclude/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/postgres-test-state-mutation-types";/,
  );
});
