import {
  collectDiagnostics,
  type CheckReport,
  type DependencyResult,
  type DiagnosticsError,
  type DiagnosticArguments,
  type DiagnosticsResult,
  type ExecutedPostgresSqlReport,
  type ImportClosureResult,
  type PostgresCompleteCatalog,
  type PostgresOrderingCatalog,
  type PostgresSqlFacts,
  type ResolvedConfig,
  type DeclaredConfig,
  type ResolveCheckResult,
  type ResolveCheckBatchResult,
  type SignatureImpactResult,
  type SymbolsResult,
} from "../../../packages/no-mistakes/index.js";

const optional: Promise<DiagnosticsResult<CheckReport>> = collectDiagnostics("check");
const required = collectDiagnostics("dependents", { files: ["src/app.ts"] });
const defaults: Promise<DiagnosticsResult<ResolvedConfig>> = collectDiagnostics("resolveConfig");
const declared: Promise<DiagnosticsResult<DeclaredConfig>> = collectDiagnostics("resolveConfig", { view: "declared" });
const symbols: Promise<DiagnosticsResult<SymbolsResult>> = collectDiagnostics("symbols", { files: [] });
const signature: Promise<DiagnosticsResult<SignatureImpactResult>> = collectDiagnostics("symbols", { files: [], mode: "signature-impact", symbol: "run" });
const paths: Promise<DiagnosticsResult<ImportClosureResult>> = collectDiagnostics("dependencies", { files: [], projection: "paths" });
const graph: Promise<DiagnosticsResult<DependencyResult>> = collectDiagnostics("dependencies", { files: [] });
const single: Promise<DiagnosticsResult<ResolveCheckResult>> = collectDiagnostics("resolveCheck", { file: "src/app.ts" });
const batch: Promise<DiagnosticsResult<ResolveCheckBatchResult>> = collectDiagnostics("resolveCheck", { files: ["src/app.ts"] });
const source = { fileName: "query.sql", sql: "SELECT 1" };
const sql: Promise<DiagnosticsResult<PostgresSqlFacts>> = collectDiagnostics("parsePostgresSql", source);
const sqlBatch: Promise<DiagnosticsResult<PostgresSqlFacts[]>> = collectDiagnostics("parsePostgresSql", [source]);
const extracted: Promise<DiagnosticsResult<ExecutedPostgresSqlReport>> = collectDiagnostics("extractExecutedPostgresSql", { sources: [], importSpecifier: "database", includeDiagnostics: true });
const ordering: Promise<DiagnosticsResult<PostgresOrderingCatalog>> = collectDiagnostics("generatePostgresCatalog", { connectionEnv: "DATABASE_URL", schema: "public", coverage: "ordering" });
const complete: Promise<DiagnosticsResult<PostgresCompleteCatalog>> = collectDiagnostics("generatePostgresCatalog", { connectionEnv: "DATABASE_URL", schema: "public" });
const error = {} as DiagnosticsError;
error.diagnostics?.hotFiles.map((file) => file.pass);
// @ts-expect-error The original required argument remains required.
collectDiagnostics("dependents");
// @ts-expect-error Live polling is independent of collection.
collectDiagnostics("diagnosticsSnapshot", { id: "run" });
// @ts-expect-error Raw Markdown bindings do not return structured diagnostics.
collectDiagnostics("testsComment", {});
// @ts-expect-error Unknown selectors are rejected.
collectDiagnostics("unknown");
void [optional, required, defaults, declared, symbols, signature, paths, graph, single, batch, sql, sqlBatch, extracted, ordering, complete];

declare const selector: "check" | "dependents";
const commonArgs: DiagnosticArguments<typeof selector> = [{ files: ["src/app.ts"] }];
void commonArgs;
collectDiagnostics(selector, { files: ["src/app.ts"] });
// @ts-expect-error Every possible selector must accept the supplied arguments.
collectDiagnostics(selector);
// @ts-expect-error Optional check options cannot satisfy the dependents branch.
collectDiagnostics(selector, {});
if (selector === "check") {
  const narrowed: Promise<DiagnosticsResult<CheckReport>> = collectDiagnostics(selector);
  void narrowed;
} else {
  collectDiagnostics(selector, { files: ["src/app.ts"] });
}
