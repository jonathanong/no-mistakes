const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("grouping advice exports named options and all five target codes", () => {
  const declaration = readFileSync(
    join(packageRoot, "postgres-grouping-not-aggregation-types.d.ts"),
    "utf8",
  );
  assert.match(declaration, /export interface PostgresGroupingNotAggregationOptions/);
  assert.match(declaration, /export type PostgresGroupingNotAggregationCode/);
  for (const code of [
    "all-of-candidate-pool",
    "distinct-over-unique-key",
    "distinct-over-union",
    "distinct-join-filter",
    "aggregate-existence-test",
  ]) {
    assert.ok(declaration.includes(`"${code}"`), code);
  }
  for (const field of [
    "schemaCatalogPath?: string;",
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
    "sqlLiteralHelpers?: string[];",
  ]) {
    assert.ok(declaration.includes(field), field);
  }
  assert.doesNotMatch(declaration, /fail\?:|sqlIncludeLocked/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export type \* from "\.\/postgres-grouping-not-aggregation-types"/,
  );
});
