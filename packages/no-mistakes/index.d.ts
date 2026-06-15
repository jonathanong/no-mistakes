import type {
  CheckReport,
  AnalyzeProjectOptions,
  AnalyzeProjectResult,
  CiEnvOptions,
  CiEnvReport,
  CiImpactOptions,
  CiImpactReport,
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
  ImpactedChecksOptions,
  ImpactedChecksReport,
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
  TestsWhyOptions,
  TraverseOptions,
  WhyStep,
} from "./types";

export * from "./types";

export function dependencies(options: TraverseOptions): Promise<DependencyResult>;
export function dependents(options: TraverseOptions): Promise<DependencyResult>;
export function related(options: TraverseOptions): Promise<DependencyResult>;
export function analyzeProject(options: AnalyzeProjectOptions): Promise<AnalyzeProjectResult>;
export function symbols(options: SymbolsSignatureImpactOptions): Promise<SignatureImpactResult>;
export function symbols(options: SymbolsListOptions): Promise<SymbolsResult>;
export function symbols(options: SymbolsOptions): Promise<SymbolsResult | SignatureImpactResult>;
export function importers(options: ImportersOptions): Promise<ImportersResult>;
export function exportsOf(options: ExportsOfOptions): Promise<ExportsOfResult>;
export function deadExports(options: DeadExportsOptions): Promise<DeadExportsResult>;
export function callSites(options: CallSitesOptions): Promise<CallSitesResult>;
export function resolveCheck(options: ResolveCheckOptions): Promise<ResolveCheckResult>;
export function fetches(options?: FetchesOptions): Promise<unknown>;
export function check(options?: ProjectOptions): Promise<CheckReport>;
export function testsPlan(options: TestsPlanOptions): Promise<TestPlan>;
export function testsImpact(options: TestsImpactOptions): Promise<TestPlan>;
export function testsWhy(options: TestsWhyOptions): Promise<Record<string, WhyStep[]>>;
export function testsComment(options: TestsPlanDocumentOptions): Promise<string>;
export function testsGraph(options: TestsPlanDocumentOptions): Promise<TestGraph>;
export function testsGraphMermaid(options: TestsPlanDocumentOptions): Promise<string>;
export function playwrightCheck(options?: PlaywrightOptions): Promise<unknown>;
export function playwrightEdges(options?: PlaywrightOptions): Promise<unknown>;
export function playwrightRelated(options: PlaywrightRelatedOptions): Promise<unknown>;
export function playwrightTests(options?: PlaywrightOptions): Promise<unknown>;
export function queues(options?: ProjectOptions): Promise<QueueReport>;
export function queueEdges(options?: ProjectOptions): Promise<GraphEdge[]>;
export function queueRelated(options: ProjectOptions): Promise<GraphEdge[]>;
export function queueCheck(options?: ProjectOptions): Promise<unknown[]>;
export function serverRoutes(options?: ProjectOptions): Promise<ServerRoutesReport>;
export function serverRouteList(options?: ProjectOptions): Promise<unknown[]>;
export function serverRouteEdges(options?: ProjectOptions): Promise<GraphEdge[]>;
export function serverRouteRelated(options: ProjectOptions): Promise<GraphEdge[]>;
export function reactAnalyze(options?: ProjectOptions): Promise<ReactComponentFacts[]>;
export function reactCheck(options?: ProjectOptions): Promise<ReactViolation[]>;
export function reactUsages(
  options: ProjectOptions & { target: string },
): Promise<ReactUsagesReport>;
export function lockfileDiff(options: LockfileDiffOptions): Promise<LockfileDiffEntry[]>;
export function ciImpact(options: CiImpactOptions): Promise<CiImpactReport>;
export function ciEnv(options: CiEnvOptions): Promise<CiEnvReport>;
export function impactedChecks(options: ImpactedChecksOptions): Promise<ImpactedChecksReport>;
export function dataPw(options: DataPwOptions): Promise<DataPwReport>;
export function effects(options: EffectsOptions): Promise<EffectsReport>;
export function rscCallers(options: RscCallersOptions): Promise<RscCallersReport>;
export function registryExtension(
  options: RegistryExtensionOptions,
): Promise<RegistryExtensionReport>;
export function infraResourceRefs(
  options: InfraOptions & { address: string },
): Promise<ResourceRefRow[]>;
export function infraOutputs(
  options: InfraOptions & { moduleDir: string },
): Promise<ModuleOutputsResult>;
export function infraTestFor(options: InfraOptions & { tfFile: string }): Promise<TestForRow[]>;
export function swiftImporters(
  options: SwiftOptions & { file: string },
): Promise<SwiftImporterRow[]>;
export function swiftTestTargets(
  options: SwiftOptions & { file: string },
): Promise<SwiftTestTargetRow[]>;
export function version(): Promise<string>;
