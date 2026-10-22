const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("mutable SQL registry configuration exposes effort and unanalyzable policy", () => {
  const types = readFileSync(join(packageRoot, "postgres-mutable-sql-registry-types.d.ts"), "utf8");
  assert.match(types, /export interface PostgresNoMutableSqlRegistryOptions/);
  assert.match(types, /effort\?: Effort;/);
  assert.match(types, /unanalyzable\?: Unanalyzable;/);
  assert.match(types, /unanalyzableSql\?: Unanalyzable;/);
  assert.match(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8"),
    /export \* from "\.\/postgres-mutable-sql-registry-types";/,
  );
});
