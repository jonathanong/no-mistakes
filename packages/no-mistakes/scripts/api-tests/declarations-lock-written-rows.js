const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("lock-written-rows options expose named checks, policies, exceptions and write sets", () => {
  const types = readFileSync(join(packageRoot, "postgres-lock-written-rows-types.d.ts"), "utf8");
  for (const name of [
    "Check",
    "Checks",
    "Severity",
    "Allow",
    "WriteSet",
    "ExecutorOption",
    "OnlyOptions",
  ])
    assert.match(types, new RegExp(`export (?:type|interface) PostgresLockWrittenRows${name}\\b`));
  for (const id of [
    "row-lock-unwritten-relation",
    "table-lock",
    "constant-advisory-lock",
    "row-lock-outside-transaction",
    "share-lock-then-write",
    "redundant-prelock",
  ])
    assert.ok(types.includes(`"${id}"`));
  assert.equal(types.match(/schemaCatalogPath\?: string;/gu)?.length, 1);
  for (const field of [
    'export type PostgresLockWrittenRowsSeverity = "fail" | "advise" | "ignore";',
    "export type PostgresLockWrittenRowsChecks = {",
    'Check extends "redundant-prelock"',
    'Exclude<PostgresLockWrittenRowsSeverity, "fail">',
    "checks?: PostgresLockWrittenRowsChecks;",
    "executorNames?: string[];",
    "transactionExecutorTypeNames?: string[];",
    "executorOptionProperties?: string[];",
    "schemaCatalogPath?: string;",
    "executorOptions?: PostgresLockWrittenRowsExecutorOption[];",
    "name: string;",
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
