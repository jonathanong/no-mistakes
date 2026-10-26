const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("executor diagnostics preserve array defaults and export a named opt-in report", () => {
  const declarations = readFileSync(join(packageRoot, "executed-postgres-sql-types.d.ts"), "utf8");
  assert.match(declarations, /includeDiagnostics\?: Diagnostics;/);
  assert.match(
    declarations,
    /export interface UndeclaredExecutorTypeSummary \{\s*type: string;\s*module: string;\s*count: number;/,
  );
  assert.match(declarations, /export interface ExecutedPostgresSqlReport \{/);
  assert.match(declarations, /statements: ExecutedPostgresSql\[\];/);
  assert.match(declarations, /undeclaredExecutorTypes: UndeclaredExecutorTypeSummary\[\];/);
  assert.match(
    declarations,
    /export type UnresolvedExecutorReceiverSummary = UndeclaredExecutorTypeSummary;/,
  );
  assert.match(declarations, /unresolvedExecutorReceivers: UnresolvedExecutorReceiverSummary\[\];/);
  assert.match(
    declarations,
    /ExecutedPostgresSqlOptions<true> & \{ includeDiagnostics: true \},\s*\): Promise<ExecutedPostgresSqlReport>;/,
  );
  assert.match(
    declarations,
    /ExecutedPostgresSqlOptions<boolean>,\s*\): Promise<ExecutedPostgresSql\[\] \| ExecutedPostgresSqlReport>;/,
  );
});
