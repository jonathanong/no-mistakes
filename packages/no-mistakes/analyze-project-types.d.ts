import type { TestsAuditOptions } from "./test-audit-types";
import type { CiTopologyOptions } from "./workflow-topology-types";
import type { CiEnvOptions, CiImpactOptions, ImpactedChecksOptions } from "./ci-types";
import type { FetchesOptions, FlowOptions } from "./flow-types";
import type { ImportUsagesOptions } from "./import-usage-types";
import type { JobsPoolWarning } from "./invocation-types";
import type { MermaidValidationOptions } from "./mermaid-types";
import type {
  DataPwOptions,
  EffectsOptions,
  EffectsReport,
  RegistryExtensionOptions,
  RscCallersOptions,
  RscCallersReport,
} from "./named-query-types";
import type {
  CallSitesOptions,
  DeadExportsOptions,
  ExportsOfOptions,
  ImportersOptions,
  ResolveCheckFilesOptions,
  ResolveCheckOptions,
} from "./query-types";
import type {
  InfraOptions,
  PlaywrightOptions,
  PlaywrightRelatedOptions,
  SwiftOptions,
} from "./report-types";
import type {
  LockfileDiffOptions,
  TestsImpactOptions,
  TestsPlanDocumentOptions,
  TestsPlanOptions,
  TestsTargetsOptions,
  TestsWhyOptions,
} from "./test-types";
import type {
  CheckOptions,
  ProjectOptions,
  SymbolsListOptions,
  SymbolsSignatureImpactOptions,
  TraverseGraphOptions,
  TraverseOptions,
  TraversePathsOptions,
} from "./traversal-types";

/** Shared `analyzeProject()` options after the request-level `root`/`tsconfig`/`config`. */
export type BatchedProjectOptions = Omit<ProjectOptions, "root" | "tsconfig" | "config">;
export type BatchedFlowOptions = Omit<FlowOptions, "root" | "tsconfig" | "config">;
export type BatchedTraverseOptions = TraverseOptions & Pick<ProjectOptions, "config">;
export type BatchedDependencyTraverseOptions = (TraverseGraphOptions | TraversePathsOptions) &
  Pick<ProjectOptions, "config">;
export type BatchedQueueRelatedOptions = BatchedProjectOptions & { files: string[] };
export type BatchedServerRouteRelatedOptions = BatchedProjectOptions &
  ({ files: string[] } | { roots: string[] });
export type BatchedReactUsagesOptions = Pick<
  ProjectOptions,
  "root" | "tsconfig" | "config" | "targets" | "include" | "effort"
> &
  Required<Pick<ProjectOptions, "target">>;
export type BatchedCheckOptions = Pick<
  CheckOptions,
  "root" | "tsconfig" | "config" | "includeSuppressed" | "effort"
> &
  Pick<CheckOptions, "includeRunnerConfigDeadlines" | "includeSqlVerdicts">;
export type BatchedQueryFileOptions<T> = Omit<T, "root" | "tsconfig">;
export type BatchedRootConfigOptions<T> = Omit<T, "root" | "config">;
export type BatchedRootTsConfigOptions<T> = Omit<T, "root" | "tsconfig" | "config">;

export type AnalyzeProjectReportRequest =
  | ({ type: "dependencies"; id?: string } & BatchedDependencyTraverseOptions)
  | ({ type: "dependents" | "related"; id?: string } & BatchedTraverseOptions)
  | ({ type: "symbols"; id?: string } & (SymbolsListOptions | SymbolsSignatureImpactOptions))
  | ({ type: "importUsages"; id?: string } & Omit<ImportUsagesOptions, "root">)
  | ({ type: "flow"; id?: string } & BatchedFlowOptions)
  | ({ type: "effects"; id?: string } & Omit<EffectsOptions, "root" | "tsconfig" | "config">)
  | ({ type: "rscCallers"; id?: string } & Omit<RscCallersOptions, "root" | "tsconfig" | "config">)
  | ({ type: "queues" | "queueEdges" | "queueCheck"; id?: string } & BatchedProjectOptions)
  | ({ type: "queueRelated"; id?: string } & BatchedQueueRelatedOptions)
  | ({
      type: "serverRoutes" | "serverRouteList" | "serverRouteEdges";
      id?: string;
    } & BatchedProjectOptions)
  | ({ type: "serverContracts"; id?: string } & BatchedProjectOptions)
  | ({ type: "serverRouteRelated"; id?: string } & BatchedServerRouteRelatedOptions)
  | ({ type: "reactAnalyze" | "reactCheck"; id?: string } & Pick<
      ProjectOptions,
      "targets" | "depth" | "assertNoFetch" | "effort"
    >)
  | ({ type: "reactUsages"; id?: string } & BatchedReactUsagesOptions)
  | ({
      type: "playwrightCheck" | "playwrightEdges" | "playwrightTests";
      id?: string;
    } & Omit<PlaywrightOptions, "root" | "config">)
  | ({ type: "playwrightRelated"; id?: string } & Omit<PlaywrightRelatedOptions, "root" | "config">)
  | ({ type: "check"; id?: string } & BatchedCheckOptions)
  | ({ type: "importers"; id?: string } & BatchedQueryFileOptions<ImportersOptions>)
  | ({ type: "exportsOf"; id?: string } & BatchedQueryFileOptions<ExportsOfOptions>)
  | ({ type: "deadExports"; id?: string } & BatchedQueryFileOptions<DeadExportsOptions>)
  | ({ type: "callSites"; id?: string } & BatchedQueryFileOptions<CallSitesOptions>)
  | ({
      type: "resolveCheck";
      id?: string;
    } & BatchedQueryFileOptions<ResolveCheckOptions | ResolveCheckFilesOptions>)
  | {
      /** Checks the union of seed and reachable files from dependency reports in this request. */
      type: "resolveCheckDependencies";
      id?: string;
      dependencyReportIds: string[];
      /** Project root. Defaults to the current working directory. */
      root?: string;
      /** Path to tsconfig.json for alias resolution. Searched upward if omitted. */
      tsconfig?: string;
      /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
      config?: string;
    }
  | ({ type: "fetches"; id?: string } & BatchedRootConfigOptions<FetchesOptions>)
  | ({ type: "dataPw"; id?: string } & BatchedRootConfigOptions<DataPwOptions>)
  | ({ type: "registryExtension"; id?: string } & Omit<RegistryExtensionOptions, "root">)
  | ({ type: "testsAudit"; id?: string } & TestsAuditOptions)
  | ({ type: "testsPlan"; id?: string } & BatchedRootTsConfigOptions<TestsPlanOptions>)
  | ({ type: "testsImpact"; id?: string } & BatchedRootTsConfigOptions<TestsImpactOptions>)
  | ({ type: "testsTargets"; id?: string } & Omit<TestsTargetsOptions, "root" | "config">)
  | ({ type: "testsWhy"; id?: string } & BatchedRootTsConfigOptions<TestsWhyOptions>)
  | ({
      type: "testsComment" | "testsGraph" | "testsGraphMermaid";
      id?: string;
    } & TestsPlanDocumentOptions)
  | ({ type: "lockfileDiff"; id?: string } & Omit<LockfileDiffOptions, "root">)
  | ({ type: "ciImpact"; id?: string } & BatchedRootConfigOptions<CiImpactOptions>)
  | ({ type: "ciEnv"; id?: string } & BatchedRootConfigOptions<CiEnvOptions>)
  | ({ type: "ciTopology"; id?: string } & BatchedRootConfigOptions<CiTopologyOptions>)
  | ({ type: "impactedChecks"; id?: string } & BatchedRootTsConfigOptions<ImpactedChecksOptions>)
  | ({
      type: "infraResourceRefs" | "infraOutputs" | "infraTestFor";
      id?: string;
    } & BatchedRootConfigOptions<InfraOptions>)
  | ({
      type: "swiftImporters" | "swiftTestTargets";
      id?: string;
    } & BatchedRootConfigOptions<SwiftOptions>)
  | ({ type: "validateMermaidMarkdown"; id?: string } & MermaidValidationOptions);

export interface AnalyzeProjectOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to tsconfig.json for alias resolution. Searched upward if omitted. */
  tsconfig?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  filters?: string[];
  reports: AnalyzeProjectReportRequest[];
}

/** Discriminant for `analyzeProject()` report requests and result entries. */
export type AnalyzeProjectReportType = AnalyzeProjectReportRequest["type"];

/** Payload carried on one `analyzeProject()` result entry. */
export type AnalyzeProjectReportResult = unknown | EffectsReport | RscCallersReport;

/** One entry in `analyzeProject()` `reports`. */
export interface AnalyzeProjectReport {
  id?: string;
  type: AnalyzeProjectReportType;
  result: AnalyzeProjectReportResult;
}

export interface AnalyzeProjectResult {
  reports: AnalyzeProjectReport[];
  /**
   * Set when this call's resolved `jobs` differs from the process-wide Rayon pool.
   * Omitted when the pool was just created or the resolved count already matches.
   * Other analysis JSON objects use the same optional field.
   */
  jobsPoolWarning?: JobsPoolWarning;
}
