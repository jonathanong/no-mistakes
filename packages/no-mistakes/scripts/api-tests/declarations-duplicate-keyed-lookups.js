const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("Duplicate keyed lookups rule exports its named option contract", () => {
  const types = readFileSync(
    join(packageRoot, "postgres-duplicate-keyed-lookups-types.d.ts"),
    "utf8",
  );
  const exports = readFileSync(join(packageRoot, "types.d.ts"), "utf8");
  assert.match(types, /export interface PostgresNoDuplicateKeyedLookupsOptions/);
  assert.match(
    types,
    /export type PostgresNoDuplicateKeyedLookupsUnanalyzableSql = "fail" \| "ignore" \| "advise";/,
  );
  for (const field of [
    "include?: string[];",
    "exclude?: string[];",
    "sqlInclude?: string[];",
    "importSpecifier?: string;",
    "executorNames?: string[];",
    "extraExecutorNames?: string[];",
    "excludeExecutorNames?: string[];",
    "executorFactoryNames?: string[];",
    "executorTypeNames?: string[];",
    "executorOptionProperties?: string[];",
    "trustedSqlTags?: { module: string; name: string }[];",
    "unanalyzable?: Unanalyzable;",
    "unanalyzableSql?: PostgresNoDuplicateKeyedLookupsUnanalyzableSql;",
  ]) {
    assert.ok(types.includes(field), `missing public option field ${field}`);
  }
  assert.match(exports, /export type \* from "\.\/postgres-duplicate-keyed-lookups-types"/);
});
