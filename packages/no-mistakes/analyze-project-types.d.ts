import type { FlowOptions } from "./flow-types";
import type { ImportUsagesOptions } from "./import-usage-types";
import type {
  EffectsOptions,
  EffectsReport,
  RscCallersOptions,
  RscCallersReport,
} from "./named-query-types";
import type { PlaywrightOptions, PlaywrightRelatedOptions } from "./report-types";
import type {
  ProjectOptions,
  SymbolsListOptions,
  SymbolsSignatureImpactOptions,
  TraverseOptions,
} from "./traversal-types";

type BatchedProjectOptions = Omit<ProjectOptions, "root" | "tsconfig" | "config">;
type BatchedFlowOptions = Omit<FlowOptions, "root" | "tsconfig" | "config">;
type BatchedTraverseOptions = TraverseOptions & Pick<ProjectOptions, "config">;
type BatchedQueueRelatedOptions = BatchedProjectOptions & { files: string[] };
type BatchedServerRouteRelatedOptions = BatchedProjectOptions &
  ({ files: string[] } | { roots: string[] });
type BatchedReactUsagesOptions = Pick<
  ProjectOptions,
  "root" | "tsconfig" | "config" | "targets" | "include"
> &
  Required<Pick<ProjectOptions, "target">>;
type BatchedCheckOptions = Pick<ProjectOptions, "root" | "tsconfig" | "config">;

export type AnalyzeProjectReportRequest =
  | ({ type: "dependencies" | "dependents" | "related"; id?: string } & BatchedTraverseOptions)
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
      "targets" | "depth" | "assertNoFetch"
    >)
  | ({ type: "reactUsages"; id?: string } & BatchedReactUsagesOptions)
  | ({
      type: "playwrightCheck" | "playwrightEdges" | "playwrightTests";
      id?: string;
    } & Omit<PlaywrightOptions, "root" | "config">)
  | ({ type: "playwrightRelated"; id?: string } & Omit<PlaywrightRelatedOptions, "root" | "config">)
  | ({ type: "check"; id?: string } & BatchedCheckOptions);

export interface AnalyzeProjectOptions {
  root?: string;
  tsconfig?: string;
  config?: string;
  filters?: string[];
  reports: AnalyzeProjectReportRequest[];
}

export interface AnalyzeProjectResult {
  reports: Array<{
    id?: string;
    type: AnalyzeProjectReportRequest["type"];
    result: unknown | EffectsReport | RscCallersReport;
  }>;
}
