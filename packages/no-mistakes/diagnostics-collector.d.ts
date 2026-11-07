import type * as Api from "./index";
import type { DiagnosticOperations, DiagnosticsResult } from "./diagnostics-types";

export function collectDiagnostics(
  operation: "dependencies",
  options: Api.WithInvocationOptions<Api.TraversePathsOptions>,
): Promise<DiagnosticsResult<Api.WithInvocationResult<Api.ImportClosureResult>>>;
export function collectDiagnostics(
  operation: "dependencies",
  options: Api.WithInvocationOptions<Api.TraverseGraphOptions>,
): Promise<DiagnosticsResult<Api.WithInvocationResult<Api.DependencyResult>>>;
export function collectDiagnostics(
  operation: "symbols",
  options: Api.WithInvocationOptions<Api.SymbolsSignatureImpactOptions>,
): Promise<DiagnosticsResult<Api.WithInvocationResult<Api.SignatureImpactResult>>>;
export function collectDiagnostics(
  operation: "symbols",
  options: Api.WithInvocationOptions<Api.SymbolsListOptions>,
): Promise<DiagnosticsResult<Api.WithInvocationResult<Api.SymbolsResult>>>;
export function collectDiagnostics(
  operation: "symbols",
  options: Api.WithInvocationOptions<Api.SymbolsOptions>,
): Promise<
  DiagnosticsResult<Api.WithInvocationResult<Api.SymbolsResult | Api.SignatureImpactResult>>
>;
export function collectDiagnostics(
  operation: "resolveCheck",
  options: Api.WithInvocationOptions<Api.ResolveCheckOptions>,
): Promise<DiagnosticsResult<Api.WithInvocationResult<Api.ResolveCheckResult>>>;
export function collectDiagnostics(
  operation: "resolveCheck",
  options: Api.WithInvocationOptions<Api.ResolveCheckFilesOptions>,
): Promise<DiagnosticsResult<Api.WithInvocationResult<Api.ResolveCheckBatchResult>>>;
export function collectDiagnostics(
  operation: "generatePostgresCatalog",
  options: Api.WithInvocationOptions<Api.PostgresCatalogOptions & { coverage: "ordering" }>,
): Promise<DiagnosticsResult<Api.WithInvocationResult<Api.PostgresOrderingCatalog>>>;
export function collectDiagnostics(
  operation: "generatePostgresCatalog",
  options: Api.WithInvocationOptions<Api.PostgresCatalogOptions & { coverage?: "complete" }>,
): Promise<DiagnosticsResult<Api.WithInvocationResult<Api.PostgresCompleteCatalog>>>;
export function collectDiagnostics(
  operation: "generatePostgresCatalog",
  options: Api.WithInvocationOptions<Api.PostgresCatalogOptions>,
): Promise<DiagnosticsResult<Api.WithInvocationResult<Api.PostgresCatalog>>>;
export function collectDiagnostics(
  operation: "resolveConfig",
  options: Api.WithInvocationOptions<Api.ConfigResolveOptions & { view: "declared" }>,
): Promise<DiagnosticsResult<Api.WithInvocationResult<Api.DeclaredConfig>>>;
export function collectDiagnostics(
  operation: "resolveConfig",
  options?: Api.WithInvocationOptions<Api.ConfigResolveOptions & { view?: "resolved" }>,
): Promise<DiagnosticsResult<Api.WithInvocationResult<Api.ResolvedConfig>>>;
export function collectDiagnostics(
  operation: "resolveConfig",
  options: Api.WithInvocationOptions<Api.ConfigResolveOptions>,
): Promise<DiagnosticsResult<Api.WithInvocationResult<Api.ResolvedConfig | Api.DeclaredConfig>>>;
export function collectDiagnostics(
  operation: "parsePostgresSql",
  source: Api.PostgresSqlSource,
): Promise<DiagnosticsResult<Api.PostgresSqlFacts>>;
export function collectDiagnostics(
  operation: "parsePostgresSql",
  sources: Api.PostgresSqlSource[],
): Promise<DiagnosticsResult<Api.PostgresSqlFacts[]>>;
export function collectDiagnostics(
  operation: "extractExecutedPostgresSql",
  options: Api.ExecutedPostgresSqlOptions<true> & { includeDiagnostics: true },
): Promise<DiagnosticsResult<Api.ExecutedPostgresSqlReport>>;
export function collectDiagnostics(
  operation: "extractExecutedPostgresSql",
  options: Api.ExecutedPostgresSqlOptions,
): Promise<DiagnosticsResult<Api.ExecutedPostgresSql[]>>;
export function collectDiagnostics(
  operation: "extractExecutedPostgresSql",
  options: Api.ExecutedPostgresSqlOptions<boolean>,
): Promise<DiagnosticsResult<Api.ExecutedPostgresSql[] | Api.ExecutedPostgresSqlReport>>;

/** Collect verbose diagnostics without stderr output or changing the operation result. */
export function collectDiagnostics<K extends keyof DiagnosticOperations>(
  operation: K,
  ...args: Parameters<DiagnosticOperations[K]>
): Promise<DiagnosticsResult<Awaited<ReturnType<DiagnosticOperations[K]>>>>;
