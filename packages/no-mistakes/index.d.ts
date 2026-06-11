import type {
  CheckReport,
  AnalyzeProjectOptions,
  AnalyzeProjectResult,
  DependencyResult,
  FetchesOptions,
  GraphEdge,
  LockfileDiffEntry,
  LockfileDiffOptions,
  PlaywrightOptions,
  PlaywrightRelatedOptions,
  ProjectOptions,
  QueueReport,
  ReactComponentFacts,
  ReactUsagesReport,
  ReactViolation,
  ServerRoutesReport,
  SignatureImpactResult,
  SymbolsOptions,
  SymbolsResult,
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
export function symbols(
  options: SymbolsOptions & { mode: "signature-impact"; symbol: string },
): Promise<SignatureImpactResult>;
export function symbols(options: SymbolsOptions): Promise<SymbolsResult>;
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
export function version(): Promise<string>;
