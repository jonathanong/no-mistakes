const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("lock-written-rows options expose named checks, policies, exceptions and write sets", () => {
  const types = readFileSync(join(packageRoot, "postgres-lock-written-rows-types.d.ts"), "utf8");
  for (const name of ["Check", "Severity", "Allow", "WriteSet", "OnlyOptions"])
    assert.match(types, new RegExp(`export (?:type|interface) PostgresLockWrittenRows${name}\\b`));
  for (const id of [
    "row-lock-unwritten-relation",
    "table-lock",
    "constant-advisory-lock",
    "row-lock-outside-transaction",
  ])
    assert.ok(types.includes(`"${id}"`));
  for (const field of [
    'export type PostgresLockWrittenRowsSeverity = "fail" | "advise" | "ignore";',
    "checks?: Partial<Record<PostgresLockWrittenRowsCheck, PostgresLockWrittenRowsSeverity>>;",
    "executorNames?: string[];",
    "executorOptionProperties?: string[];",
    "extraExecutorNames?: string[];",
    "excludeExecutorNames?: string[];",
    "allow?: PostgresLockWrittenRowsAllow[];",
    "writeSets?: PostgresLockWrittenRowsWriteSet[];",
    "transactionLine: number;",
    "relations: string[];",
    "reason: string;",
    'unanalyzableSql?: "fail" | "ignore" | "advise";',
  ])
    assert.ok(types.includes(field), field);
  assert.ok(
    readFileSync(join(packageRoot, "types.d.ts"), "utf8").includes(
      'export type * from "./postgres-lock-written-rows-types";',
    ),
  );
});
