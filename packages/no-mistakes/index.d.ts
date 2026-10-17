import type { TypeScriptModulesOptions, TypeScriptModulesReport } from "./typescript-module-types";
export type * from "./typescript-module-types";
/** Analyze selected TS/JS modules asynchronously with source-local binding semantics. */

import type { PostgresSqlSource, PostgresSqlFacts } from "./postgres-source-types";
export { extractExecutedPostgresSql } from "./executed-postgres-sql-types";
export * from "./postgres-source-types";
/** Parse SQL on the native async worker without repository I/O or a database. */
export function parsePostgresSql(source: PostgresSqlSource): Promise<PostgresSqlFacts>;
export function parsePostgresSql(sources: PostgresSqlSource[]): Promise<PostgresSqlFacts[]>;
import type {
  CheckReport,
  AnalyzeProjectOptions,
  AnalyzeProjectResult,
  CallSitesOptions,
  CallSitesResult,
  CheckOptions,
  DeadExportsOptions,
  DeadExportsResult,
  DependencyResult,
  ExportsOfOptions,
  ExportsOfResult,
  FetchReport,
  FetchesOptions,
  FlowOptions,
  FlowReport,
  ImportUsagesOptions,
  ImportUsagesResult,
  ImportersOptions,
  ImportersResult,
  MermaidValidationOptions,
  MermaidValidationResult,
  ResolveCheckOptions,
  ResolveCheckFilesOptions,
  ResolveCheckResult,
  ResolveCheckBatchResult,
  ResolvedConfig,
  GraphEdge,
  PlaywrightOptions,
  PlaywrightRelatedOptions,
  ProjectOptions,
  QueueCheckFinding,
  QueueEdge,
  QueueReport,
  ReactComponentFacts,
  ReactUsagesReport,
  ReactViolation,
  ServerRoutesReport,
  ServerContractsReport,
  SignatureImpactResult,
  SymbolsListOptions,
  SymbolsOptions,
  SymbolsResult,
  SymbolsSignatureImpactOptions,
  TestGraph,
  TestPlan,
  TestsImpactOptions,
  TestsPlanDocumentOptions,
  TestsPlanOptions,
  TestsTargetsOptions,
  TestsTargetsReport,
  TestsWhyOptions,
  TraverseGraphOptions,
  TraverseOptions,
  TraversePathsOptions,
  ImportClosureResult,
  WhyStep,
  WithInvocationOptions,
  WithInvocationResult,
  WritePlanningImpactArtifactsOptions,
  PlanningImpactArtifacts,
} from "./types";

export * from "./types";
export * from "./index-ci-infra";

export function dependencies(
  options: WithInvocationOptions<TraversePathsOptions>,
): Promise<WithInvocationResult<ImportClosureResult>>;
export function dependencies(
  options: WithInvocationOptions<TraverseGraphOptions>,
): Promise<WithInvocationResult<DependencyResult>>;
export function dependents(
  options: WithInvocationOptions<TraverseOptions>,
): Promise<WithInvocationResult<DependencyResult>>;
export function related(
  options: WithInvocationOptions<TraverseOptions>,
): Promise<WithInvocationResult<DependencyResult>>;
export function analyzeProject(
  options: WithInvocationOptions<AnalyzeProjectOptions>,
): Promise<WithInvocationResult<AnalyzeProjectResult>>;
export function writePlanningImpactArtifacts(
  options: WithInvocationOptions<WritePlanningImpactArtifactsOptions>,
): Promise<WithInvocationResult<PlanningImpactArtifacts>>;
export function symbols(
  options: WithInvocationOptions<SymbolsSignatureImpactOptions>,
): Promise<WithInvocationResult<SignatureImpactResult>>;
export function symbols(
  options: WithInvocationOptions<SymbolsListOptions>,
): Promise<WithInvocationResult<SymbolsResult>>;
export function symbols(
  options: WithInvocationOptions<SymbolsOptions>,
): Promise<WithInvocationResult<SymbolsResult | SignatureImpactResult>>;
export function importUsages(
  options?: WithInvocationOptions<ImportUsagesOptions>,
): Promise<WithInvocationResult<ImportUsagesResult>>;
export function importers(
  options: WithInvocationOptions<ImportersOptions>,
): Promise<WithInvocationResult<ImportersResult>>;
export function exportsOf(
  options: WithInvocationOptions<ExportsOfOptions>,
): Promise<WithInvocationResult<ExportsOfResult>>;
export function deadExports(
  options: WithInvocationOptions<DeadExportsOptions>,
): Promise<WithInvocationResult<DeadExportsResult>>;
export function callSites(
  options: WithInvocationOptions<CallSitesOptions>,
): Promise<WithInvocationResult<CallSitesResult>>;
export function resolveCheck(
  options: WithInvocationOptions<ResolveCheckOptions>,
): Promise<WithInvocationResult<ResolveCheckResult>>;
export function resolveCheck(
  options: WithInvocationOptions<ResolveCheckFilesOptions>,
): Promise<WithInvocationResult<ResolveCheckBatchResult>>;
export function fetches(
  options?: WithInvocationOptions<FetchesOptions>,
): Promise<WithInvocationResult<FetchReport>>;
export function flow(
  options: WithInvocationOptions<FlowOptions>,
): Promise<WithInvocationResult<FlowReport>>;
export function check(
  options?: WithInvocationOptions<CheckOptions>,
): Promise<WithInvocationResult<CheckReport>>;
export function resolveConfig(
  options?: WithInvocationOptions<ProjectOptions>,
): Promise<WithInvocationResult<ResolvedConfig>>;
export function validateMermaidMarkdown(
  options: WithInvocationOptions<MermaidValidationOptions>,
): Promise<WithInvocationResult<MermaidValidationResult>>;
export function testsPlan(
  options: WithInvocationOptions<TestsPlanOptions>,
): Promise<WithInvocationResult<TestPlan>>;
export function testsImpact(
  options: WithInvocationOptions<TestsImpactOptions>,
): Promise<WithInvocationResult<TestPlan>>;
export function testsTargets(
  options: WithInvocationOptions<TestsTargetsOptions>,
): Promise<WithInvocationResult<TestsTargetsReport>>;
export function testsWhy(
  options: WithInvocationOptions<TestsWhyOptions>,
): Promise<WithInvocationResult<Record<string, WhyStep[]>>>;
export function testsComment(
  options: WithInvocationOptions<TestsPlanDocumentOptions>,
): Promise<string>;
export function testsGraph(
  options: WithInvocationOptions<TestsPlanDocumentOptions>,
): Promise<WithInvocationResult<TestGraph>>;
export function testsGraphMermaid(
  options: WithInvocationOptions<TestsPlanDocumentOptions>,
): Promise<string>;
export function playwrightCheck(
  options?: WithInvocationOptions<PlaywrightOptions>,
): Promise<unknown>;
export function playwrightEdges(
  options?: WithInvocationOptions<PlaywrightOptions>,
): Promise<unknown>;
export function playwrightRelated(
  options: WithInvocationOptions<PlaywrightRelatedOptions>,
): Promise<unknown>;
export function playwrightTests(
  options?: WithInvocationOptions<PlaywrightOptions>,
): Promise<unknown>;
export function queues(
  options?: WithInvocationOptions<ProjectOptions>,
): Promise<WithInvocationResult<QueueReport>>;
export function queueEdges(options?: WithInvocationOptions<ProjectOptions>): Promise<QueueEdge[]>;
export function queueRelated(options: WithInvocationOptions<ProjectOptions>): Promise<QueueEdge[]>;
export function queueCheck(
  options?: WithInvocationOptions<ProjectOptions>,
): Promise<QueueCheckFinding[]>;
export function serverRoutes(
  options?: WithInvocationOptions<ProjectOptions>,
): Promise<WithInvocationResult<ServerRoutesReport>>;
export function serverRouteList(
  options?: WithInvocationOptions<ProjectOptions>,
): Promise<unknown[]>;
export function serverRouteEdges(
  options?: WithInvocationOptions<ProjectOptions>,
): Promise<GraphEdge[]>;
export function serverRouteRelated(
  options: WithInvocationOptions<ProjectOptions>,
): Promise<GraphEdge[]>;
export function serverContracts(
  options?: WithInvocationOptions<ProjectOptions>,
): Promise<WithInvocationResult<ServerContractsReport>>;
export function reactAnalyze(
  options?: WithInvocationOptions<ProjectOptions>,
): Promise<ReactComponentFacts[]>;
export function reactCheck(
  options?: WithInvocationOptions<ProjectOptions>,
): Promise<ReactViolation[]>;
export function reactUsages(
  options: WithInvocationOptions<ProjectOptions & { target: string }>,
): Promise<WithInvocationResult<ReactUsagesReport>>;
