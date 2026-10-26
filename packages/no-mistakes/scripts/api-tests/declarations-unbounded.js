const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("unbounded SQL identifier options expose shared analysis policy", () => {
  const types = readFileSync(join(packageRoot, "postgres-unbounded-identifier-types.d.ts"), "utf8");
  assert.match(types, /export interface PostgresNoUnboundedSqlIdentifierOptions/);
  for (const field of [
    "include",
    "exclude",
    "executorNames",
    "executorFactoryNames",
    "executorTypeNames",
  ]) {
    assert.ok(types.includes(`${field}?: string[];`));
  }
  assert.match(types, /importSpecifier\?: string;/);
  assert.match(types, /effort\?: Effort;/);
  assert.match(types, /unanalyzable\?: Unanalyzable;/);
  assert.match(types, /unanalyzableSql\?: Unanalyzable;/);
  assert.match(types, /trustedSqlTags\?: \{ module: string; name: string \}\[\];/);
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      'export type * from "./postgres-unbounded-identifier-types";',
    ),
  );
});
