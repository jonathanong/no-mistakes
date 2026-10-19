const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("TypeScript module facts expose named contracts and async invocation options", () => {
  const index =
    readFileSync(join(packageRoot, "index.d.ts"), "utf8") +
    readFileSync(join(packageRoot, "index-ci-infra.d.ts"), "utf8");
  const types = readFileSync(join(packageRoot, "typescript-module-types.d.ts"), "utf8");
  assert.match(index, /export type \* from "\.\/typescript-module-types"/);
  assert.match(
    index,
    /options: WithInvocationOptions<TypeScriptModulesOptions>,\n\): Promise<WithInvocationResult<TypeScriptModulesReport>>/,
  );
  for (const name of [
    "ModulesOptions",
    "ModuleSpan",
    "BindingReference",
    "ModuleBinding",
    "ModuleScope",
    "ImportBindingKind",
    "ModuleImportBinding",
    "ModuleImport",
    "ModuleExport",
    "ModuleLoadKind",
    "ModuleLoad",
    "ModuleDiagnosticKind",
    "ModuleDiagnostic",
    "ModuleFacts",
    "ModulesReport",
  ]) {
    assert.match(types, new RegExp(`export (?:interface|type) TypeScript${name}\\b`));
  }
});

test("nested DO constraints remain typed through procedural declaration paths", () => {
  const source = readFileSync(join(packageRoot, "postgres-source-types.d.ts"), "utf8");
  const procedural = readFileSync(join(packageRoot, "postgres-procedural-types.d.ts"), "utf8");
  assert.match(source, /kind: "doBlock"; block: PostgresSqlProceduralBlock/);
  assert.match(source, /kind: "conditional";\s+branches: PostgresSqlConditionalBranch\[\]/);
  assert.match(procedural, /statements: PostgresSqlStatement\[\];/);
  assert.match(
    source,
    /kind: "addConstraint"; constraint: PostgresSqlConstraint; notValid: boolean/,
  );
  assert.match(source, /referencedTable: PostgresSqlName \| null;/);
  assert.match(source, /referencedColumns: PostgresSqlIdentifier\[\];/);
  assert.match(procedural, /spans retain original global source coordinates/);
});

test("data-modifying CTE facts expose named exported declaration contracts", () => {
  const query = readFileSync(join(packageRoot, "postgres-query-types.d.ts"), "utf8");
  const child = readFileSync(join(packageRoot, "postgres-query-statement-types.d.ts"), "utf8");
  assert.match(query, /export type \* from "\.\/postgres-query-statement-types"/);
  assert.match(query, /nestedStatements: PostgresSqlQueryStatement\[\];/);
  for (const name of [
    "QueryStatementProvenance",
    "CteInsert",
    "CteUpdate",
    "CteDelete",
    "CteMerge",
    "DmlAssignment",
    "MergeClause",
  ])
    assert.match(child, new RegExp(`export interface PostgresSql${name} [{]`));
  for (const name of [
    "QueryStatement",
    "QueryStatementKind",
    "CteInsertSource",
    "MergeAction",
    "ReturningItem",
  ])
    assert.match(child, new RegExp(`export type PostgresSql${name} =`));
  for (const field of [
    "cteId",
    "queryScopeId",
    "parentScopeId",
    "sql",
    "span",
    "returning",
    "complete",
    "unsupported",
  ])
    assert.match(child, new RegExp(`\\b${field}:`));
  assert.match(child, /onConflict: PostgresSqlConflict \| null;/);
  assert.match(child, /columnSources\?: PostgresSqlInsertColumnSources/);
  assert.match(child, /\{ kind: "select"; queryScopeId: number; span: PostgresSqlSpan \| null \}/);
});

test("SQL wrappers expose named execution and child-source contracts", () => {
  const wrappers = readFileSync(join(packageRoot, "postgres-wrapper-types.d.ts"), "utf8");
  const source = readFileSync(join(packageRoot, "postgres-source-types.d.ts"), "utf8");
  const functions = readFileSync(join(packageRoot, "postgres-function-types.d.ts"), "utf8");
  assert.match(
    wrappers,
    /export type PostgresSqlExecution = "nonExecuting" \| "executesForAnalysis" \| "unknown"/,
  );
  assert.match(
    wrappers,
    /export type PostgresSqlWrapperKind = "explain" \| "prepare" \| "functionDeclaration"/,
  );
  assert.match(wrappers, /export interface PostgresSqlWrapper/);
  for (const field of [
    "wrapperKind: PostgresSqlWrapperKind",
    "execution: PostgresSqlExecution",
    "statements: PostgresSqlStatement[]",
    "span: PostgresSqlSpan | null",
    "complete: boolean",
    "diagnostics: PostgresSqlDiagnostic[]",
  ]) {
    assert.ok(wrappers.includes(field));
  }
  assert.match(source, /export type \* from "\.\/postgres-wrapper-types"/);
  assert.match(source, /kind: "wrapper"; wrapper: PostgresSqlWrapper/);
  assert.match(functions, /wrapper: PostgresSqlWrapper/);
});

test("literal EXECUTE contracts export decoded SQL provenance", () => {
  const source = readFileSync(join(packageRoot, "postgres-source-types.d.ts"), "utf8");
  const procedural = readFileSync(join(packageRoot, "postgres-procedural-types.d.ts"), "utf8");
  assert.match(source, /kind: "literalExecute"; execute: PostgresSqlLiteralExecute/);
  assert.match(procedural, /export interface PostgresSqlLiteralExecute/);
  assert.match(procedural, /decodedSql: string;/);
  assert.match(procedural, /literalSpan: PostgresSqlSpan;/);
  assert.match(procedural, /using: PostgresSqlExpression\[\];/);
  assert.match(
    procedural,
    /export type PostgresSqlExecuteEncoding = PostgresSqlBodyEncoding \| "concatenated";/,
  );
  assert.match(procedural, /bodyEncoding: PostgresSqlExecuteEncoding;/);
});

test("structured trigger events and cast types remain named public contracts", () => {
  const source = readFileSync(join(packageRoot, "postgres-source-types.d.ts"), "utf8");
  const triggers = readFileSync(join(packageRoot, "postgres-trigger-types.d.ts"), "utf8");
  assert.match(triggers, /export interface PostgresSqlTrigger \{/);
  assert.match(source, /export type \* from "\.\/postgres-trigger-types"/);
  const expressions = readFileSync(join(packageRoot, "postgres-expression-types.d.ts"), "utf8");
  assert.match(
    triggers,
    /export type PostgresSqlTriggerEventKind = "insert" \| "delete" \| "truncate" \| "update";/,
  );
  assert.match(triggers, /export interface PostgresSqlTriggerEvent/);
  assert.match(triggers, /kind: PostgresSqlTriggerEventKind;/);
  assert.match(triggers, /updateOf: boolean;/);
  assert.match(triggers, /updateColumns: PostgresSqlIdentifier\[\];/);
  assert.match(triggers, /eventFacts: PostgresSqlTriggerEvent\[\];/);
  assert.match(expressions, /dataTypeFacts: PostgresSqlType;/);
  assert.match(
    expressions,
    /kind: "cast"; castKind: string; dataType: string; dataTypeFacts: PostgresSqlType/,
  );
});
