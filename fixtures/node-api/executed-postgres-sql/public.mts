import {
  extractExecutedPostgresSql,
  type ExecutedPostgresSql,
  type ExecutedPostgresSqlKind,
  type ExecutedPostgresSqlReport,
  type UndeclaredExecutorTypeSummary,
  type ExecutedPostgresSqlOptions,
  type ExecutedPostgresResolvedImport,
  type ExecutedPostgresTrustedSqlTag,
  type PostgresCatalog,
} from "no-mistakes";

const trustedTag: ExecutedPostgresTrustedSqlTag = { module: "@db", name: "sql" };
const edge: ExecutedPostgresResolvedImport = {
  fromFileName: "query.ts",
  specifier: "@builder",
  toFileName: "builder.ts",
};
const options: ExecutedPostgresSqlOptions = {
  sources: [
    { fileName: "query.ts", source: "query('SELECT 1')" },
    { fileName: "builder.ts", source: "export const sql = 'SELECT 1'" },
  ],
  importSpecifier: "@db",
  trustedSqlTags: [trustedTag],
  resolveImports: true,
  resolvedImports: [edge],
};
const result: Promise<ExecutedPostgresSql[]> = extractExecutedPostgresSql(options);
// Catalog types must remain available from the package root after moving their barrel export.
const catalog: PostgresCatalog | undefined = undefined;
void catalog;

result.then((queries) => {
  const kind: ExecutedPostgresSqlKind = queries[0]!.kind;
  const reason: string | null = queries[0]!.incompleteReason;
  void kind;
  void reason;
});

const report: Promise<ExecutedPostgresSqlReport> = extractExecutedPostgresSql({
  ...options,
  includeDiagnostics: true,
});
const legacyArray: Promise<ExecutedPostgresSql[]> = extractExecutedPostgresSql({
  ...options,
  includeDiagnostics: false,
});
const dynamicOptions: ExecutedPostgresSqlOptions<boolean> = {
  ...options,
  includeDiagnostics: Math.random() > 0.5,
};
const dynamic: Promise<ExecutedPostgresSql[] | ExecutedPostgresSqlReport> =
  extractExecutedPostgresSql(dynamicOptions);
report.then((value) => {
  const summary: UndeclaredExecutorTypeSummary[] = value.undeclaredExecutorTypes;
  const statements: ExecutedPostgresSql[] = value.statements;
  void summary;
  void statements;
});
void legacyArray;
void dynamic;
