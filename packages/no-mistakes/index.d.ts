import type {
  CheckReport,
  AnalyzeProjectOptions,
  AnalyzeProjectResult,
  CiEnvOptions,
  CiEnvReport,
  CiImpactOptions,
  CiImpactReport,
  CiTopologyOptions,
  WorkflowTopology,
  WorkflowTopologyIndex,
  CallSitesOptions,
  CallSitesResult,
  DataPwOptions,
  DataPwReport,
  DeadExportsOptions,
  DeadExportsResult,
  EffectsOptions,
  EffectsReport,
  RscCallersOptions,
  RscCallersReport,
  RegistryExtensionOptions,
  RegistryExtensionReport,
  DependencyResult,
  ExportsOfOptions,
  ExportsOfResult,
  FetchesOptions,
  FlowOptions,
  FlowReport,
  ImpactedChecksOptions,
  ImpactedChecksReport,
  ImportUsagesOptions,
  ImportUsagesResult,
  ImportersOptions,
  ImportersResult,
  ResolveCheckOptions,
  ResolveCheckResult,
  GraphEdge,
  InfraOptions,
  LockfileDiffEntry,
  LockfileDiffOptions,
  ModuleOutputsResult,
  PlaywrightOptions,
  PlaywrightRelatedOptions,
  ProjectOptions,
  QueueReport,
  ReactComponentFacts,
  ReactUsagesReport,
  ReactViolation,
  ResourceRefRow,
  ServerRoutesReport,
  ServerContractsReport,
  SignatureImpactResult,
  SwiftImporterRow,
  SwiftOptions,
  SwiftTestTargetRow,
  TestForRow,
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
  TraverseOptions,
  WhyStep,
  WithInvocationOptions,
} from "./types";

export * from "./types";

export function dependencies(
  options: WithInvocationOptions<TraverseOptions>,
): Promise<DependencyResult>;
export function dependents(
  options: WithInvocationOptions<TraverseOptions>,
): Promise<DependencyResult>;
export function related(options: WithInvocationOptions<TraverseOptions>): Promise<DependencyResult>;
export function analyzeProject(
  options: WithInvocationOptions<AnalyzeProjectOptions>,
): Promise<AnalyzeProjectResult>;
export function symbols(
  options: WithInvocationOptions<SymbolsSignatureImpactOptions>,
): Promise<SignatureImpactResult>;
export function symbols(options: WithInvocationOptions<SymbolsListOptions>): Promise<SymbolsResult>;
export function symbols(
  options: WithInvocationOptions<SymbolsOptions>,
): Promise<SymbolsResult | SignatureImpactResult>;
export function importUsages(
  options?: WithInvocationOptions<ImportUsagesOptions>,
): Promise<ImportUsagesResult>;
export function importers(
  options: WithInvocationOptions<ImportersOptions>,
): Promise<ImportersResult>;
export function exportsOf(
  options: WithInvocationOptions<ExportsOfOptions>,
): Promise<ExportsOfResult>;
export function deadExports(
  options: WithInvocationOptions<DeadExportsOptions>,
): Promise<DeadExportsResult>;
export function callSites(
  options: WithInvocationOptions<CallSitesOptions>,
): Promise<CallSitesResult>;
export function resolveCheck(
  options: WithInvocationOptions<ResolveCheckOptions>,
): Promise<ResolveCheckResult>;
export function fetches(options?: WithInvocationOptions<FetchesOptions>): Promise<unknown>;
export function flow(options: WithInvocationOptions<FlowOptions>): Promise<FlowReport>;
export function check(options?: WithInvocationOptions<ProjectOptions>): Promise<CheckReport>;
export function testsPlan(options: WithInvocationOptions<TestsPlanOptions>): Promise<TestPlan>;
export function testsImpact(options: WithInvocationOptions<TestsImpactOptions>): Promise<TestPlan>;
export function testsTargets(
  options: WithInvocationOptions<TestsTargetsOptions>,
): Promise<TestsTargetsReport>;
export function testsWhy(
  options: WithInvocationOptions<TestsWhyOptions>,
): Promise<Record<string, WhyStep[]>>;
export function testsComment(
  options: WithInvocationOptions<TestsPlanDocumentOptions>,
): Promise<string>;
export function testsGraph(
  options: WithInvocationOptions<TestsPlanDocumentOptions>,
): Promise<TestGraph>;
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
export function queues(options?: WithInvocationOptions<ProjectOptions>): Promise<QueueReport>;
export function queueEdges(options?: WithInvocationOptions<ProjectOptions>): Promise<GraphEdge[]>;
export function queueRelated(options: WithInvocationOptions<ProjectOptions>): Promise<GraphEdge[]>;
export function queueCheck(options?: WithInvocationOptions<ProjectOptions>): Promise<unknown[]>;
export function serverRoutes(
  options?: WithInvocationOptions<ProjectOptions>,
): Promise<ServerRoutesReport>;
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
): Promise<ServerContractsReport>;
export function reactAnalyze(
  options?: WithInvocationOptions<ProjectOptions>,
): Promise<ReactComponentFacts[]>;
export function reactCheck(
  options?: WithInvocationOptions<ProjectOptions>,
): Promise<ReactViolation[]>;
export function reactUsages(
  options: WithInvocationOptions<ProjectOptions & { target: string }>,
): Promise<ReactUsagesReport>;
export function lockfileDiff(
  options: WithInvocationOptions<LockfileDiffOptions>,
): Promise<LockfileDiffEntry[]>;
export function ciImpact(options: WithInvocationOptions<CiImpactOptions>): Promise<CiImpactReport>;
export function ciEnv(options: WithInvocationOptions<CiEnvOptions>): Promise<CiEnvReport>;
export function ciTopology(
  options?: WithInvocationOptions<CiTopologyOptions>,
): Promise<WorkflowTopology>;
/** Builds a query index over a `ciTopology()` result. Pure JS — never crosses the N-API boundary. */
export function createWorkflowTopologyIndex(topology: WorkflowTopology): WorkflowTopologyIndex;
export function impactedChecks(
  options: WithInvocationOptions<ImpactedChecksOptions>,
): Promise<ImpactedChecksReport>;
export function dataPw(options: WithInvocationOptions<DataPwOptions>): Promise<DataPwReport>;
export function effects(options: WithInvocationOptions<EffectsOptions>): Promise<EffectsReport>;
export function rscCallers(
  options: WithInvocationOptions<RscCallersOptions>,
): Promise<RscCallersReport>;
export function registryExtension(
  options: WithInvocationOptions<RegistryExtensionOptions>,
): Promise<RegistryExtensionReport>;
export function infraResourceRefs(
  options: WithInvocationOptions<InfraOptions & { address: string }>,
): Promise<ResourceRefRow[]>;
export function infraOutputs(
  options: WithInvocationOptions<InfraOptions & { moduleDir: string }>,
): Promise<ModuleOutputsResult>;
export function infraTestFor(
  options: WithInvocationOptions<InfraOptions & { tfFile: string }>,
): Promise<TestForRow[]>;
export function swiftImporters(
  options: WithInvocationOptions<SwiftOptions & { file: string }>,
): Promise<SwiftImporterRow[]>;
export function swiftTestTargets(
  options: WithInvocationOptions<SwiftOptions & { file: string }>,
): Promise<SwiftTestTargetRow[]>;
export function version(): Promise<string>;
