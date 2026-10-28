const { assert, test, readFileSync, join, packageRoot } = require("./context");

test("graph declarations expose tRPC relationships and virtual nodes", () => {
  const traversalDeclarations = readFileSync(join(packageRoot, "traversal-types.d.ts"), "utf8");
  const flowDeclarations = readFileSync(join(packageRoot, "flow-types.d.ts"), "utf8");

  assert.match(traversalDeclarations, /\| "trpc"/);
  assert.match(traversalDeclarations, /routerFile\?: string;/);
  assert.match(traversalDeclarations, /procedure\?: string;/);
  assert.match(flowDeclarations, /"trpc-procedure"/);
  assert.match(flowDeclarations, /routerFile\?: string;/);
  assert.match(flowDeclarations, /procedure\?: string;/);
});

test("graph declarations expose opt-in call relationships", () => {
  const declarations = readFileSync(join(packageRoot, "traversal-types.d.ts"), "utf8");
  assert.match(declarations, /\| "call"/);
});

test("dependency declarations expose bounded import-closure inputs and compact output", () => {
  const traversalDeclarations = readFileSync(join(packageRoot, "traversal-types.d.ts"), "utf8");
  const closureDeclarations = readFileSync(join(packageRoot, "import-closure-types.d.ts"), "utf8");
  const indexDeclarations = readFileSync(join(packageRoot, "index.d.ts"), "utf8");
  assert.doesNotMatch(traversalDeclarations, /candidateInclude\?: string\[\];/);
  assert.match(closureDeclarations, /candidateInclude\?: string\[\];/);
  assert.match(closureDeclarations, /candidateExclude\?: string\[\];/);
  assert.match(closureDeclarations, /projection\?: "graph";/);
  assert.match(closureDeclarations, /projection: "paths";/);
  assert.match(closureDeclarations, /export type TraverseProjection = "graph" \| "paths";/);
  assert.match(
    closureDeclarations,
    /export interface ImportClosureResult \{\n  files: string\[\];\n  diagnostics: TsConfigDiagnostic\[\];\n\}/,
  );
  assert.match(
    indexDeclarations,
    /options: WithInvocationOptions<TraversePathsOptions>,\n\): Promise<WithInvocationResult<ImportClosureResult>>;/,
  );
  assert.match(
    indexDeclarations,
    /options: WithInvocationOptions<TraverseGraphOptions>,\n\): Promise<WithInvocationResult<DependencyResult>>;/,
  );
  assert.doesNotMatch(indexDeclarations, /TraverseGraphOptions \| TraverseOptions/);
});

test("declarations expose invocation controls on every analysis", () => {
  const indexDeclarations = readFileSync(join(packageRoot, "index.d.ts"), "utf8");
  const invocationDeclarations = readFileSync(join(packageRoot, "invocation-types.d.ts"), "utf8");

  assert.match(invocationDeclarations, /timeout\?: number \| null;/);
  assert.match(invocationDeclarations, /invocationTimeoutMs\?: number;/);
  assert.match(invocationDeclarations, /lockTimeout\?: number \| null;/);
  assert.match(invocationDeclarations, /failOnLock\?: boolean;/);
  assert.match(invocationDeclarations, /jobs\?: number \| null;/);
  assert.match(
    invocationDeclarations,
    /current user's invocation lock \(shared across project roots\)/,
  );
  assert.match(
    invocationDeclarations,
    /positive integer takes precedence over `RAYON_NUM_THREADS`/,
  );
  assert.match(invocationDeclarations, /Omit, `0`, or `null` uses `RAYON_NUM_THREADS`/);
  assert.match(
    invocationDeclarations,
    /acquires the invocation lock initializes the process-wide pool before/,
  );
  assert.match(invocationDeclarations, /command-specific options are validated/);
  assert.match(invocationDeclarations, /JSON\/control or lock failures cannot/);
  assert.match(invocationDeclarations, /Later calls cannot resize an already initialized pool/);
  assert.match(invocationDeclarations, /export interface JobsPoolWarning/);
  assert.match(invocationDeclarations, /code: "rayon-pool-size"/);
  assert.match(invocationDeclarations, /requestedJobs: number/);
  assert.match(invocationDeclarations, /effectiveJobs: number/);
  assert.match(invocationDeclarations, /jobsPoolWarning/);
  assert.match(invocationDeclarations, /each queued run uses the full pool/);
  assert.match(
    readFileSync(join(packageRoot, "analyze-project-types.d.ts"), "utf8"),
    /jobsPoolWarning\?: JobsPoolWarning/,
  );
  assert.match(
    indexDeclarations,
    /analyzeProject\(\n  options: WithInvocationOptions<AnalyzeProjectOptions>/,
  );

  const declarations = indexDeclarations.matchAll(
    /export function (\w+)\([\s\S]*?\): Promise<[^;]+>;/g,
  );
  for (const [declaration, name] of declarations) {
    if (
      name === "version" ||
      name === "parsePostgresSql" ||
      name === "diagnosticsSnapshot" ||
      name === "lockStatus"
    ) {
      assert.doesNotMatch(declaration, /WithInvocationOptions/);
    } else {
      assert.match(declaration, /WithInvocationOptions/, `${name} must accept invocation options`);
    }
  }
});

test("SELECT scope facts expose named public contracts", () => {
  const source = readFileSync(join(packageRoot, "postgres-source-types.d.ts"), "utf8");
  const query = readFileSync(join(packageRoot, "postgres-query-types.d.ts"), "utf8");
  assert.match(source, /kind: "select"; query: PostgresSqlQuery/);
  assert.match(source, /export type \* from "\.\/postgres-query-types"/);
  for (const name of [
    "Query",
    "QueryScope",
    "QueryRelation",
    "QueryJoin",
    "QueryColumn",
    "QueryEquality",
    "QueryExists",
    "QueryCte",
    "QueryUnsupported",
    "PredicateContext",
  ]) {
    assert.match(query, new RegExp(`export interface PostgresSql${name} [{]`));
  }
  for (const name of ["Clause", "RelationKind", "JoinKind", "ColumnResolution"])
    assert.match(query, new RegExp(`export type PostgresSqlQuery${name} =`));
  assert.match(query, /unsupported: PostgresSqlQueryUnsupported\[\];/);
  assert.match(
    query,
    /export interface PostgresSqlQueryScope \{[^}]*span: PostgresSqlSpan \| null;/,
  );
  assert.match(query, /correlations: PostgresSqlQueryColumn\[\];/);
  assert.match(query, /notDepth: number;/);
  assert.match(query, /effectiveNegated: boolean \| null;/);
  assert.match(query, /effectiveMandatory: boolean \| null;/);
});

test("INSERT facts expose named exported declaration contracts", () => {
  const declarations = readFileSync(join(packageRoot, "postgres-source-types.d.ts"), "utf8");
  const insert = readFileSync(join(packageRoot, "postgres-insert-types.d.ts"), "utf8");
  assert.match(declarations, /kind: "insert"; insert: PostgresSqlInsert/);
  assert.match(declarations, /export type [*] from "\.\/postgres-insert-types"/);
  for (const name of [
    "Insert",
    "Conflict",
    "InsertAssignment",
    "AssignmentTarget",
    "ArbiterOperatorClass",
    "ArbiterParameter",
  ])
    assert.match(insert, new RegExp(`export interface PostgresSql${name} [{]`));
  for (const name of [
    "InsertSource",
    "ConflictTarget",
    "ConflictAction",
    "InsertProvenance",
    "AssignmentStep",
  ])
    assert.match(insert, new RegExp(`export type PostgresSql${name} =`));
  assert.match(insert, /target\?: PostgresSqlAssignmentTarget/);
  assert.match(insert, /kind: "expressions";\s*expressions: PostgresSqlExpression\[\]/);
  assert.match(insert, /subscripts: PostgresSqlExpression\[\]/);
  assert.match(insert, /parameters\?: PostgresSqlArbiterParameter\[\]/);
  assert.match(insert, /indirection\?: PostgresSqlAssignmentStep\[\]/);
  assert.match(insert, /operatorClasses\?: \(PostgresSqlArbiterOperatorClass \| null\)\[\]/);
  assert.match(insert, /columnsOmitted: boolean/);
  assert.match(insert, /returning: PostgresSqlReturningItem\[\]/);
  assert.match(insert, /\| "derived"/);
  const expressions = readFileSync(join(packageRoot, "postgres-expression-types.d.ts"), "utf8");
  assert.match(
    expressions,
    /kind: "unary"; operator: string; expression: PostgresSqlExpressionRoot/,
  );
  assert.match(insert, /diagnostics: PostgresSqlDiagnostic\[\]/);
});
