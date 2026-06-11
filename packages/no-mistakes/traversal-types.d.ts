import type { PlaywrightOptions, PlaywrightRelatedOptions } from "./report-types";

export type Relationship =
  | "import"
  | "import-static"
  | "import-dynamic"
  | "import-type"
  | "import-require"
  | "workspace"
  | "package"
  | "test"
  | "route"
  | "queue"
  | "md"
  | "ci"
  | "http"
  | "process"
  | "asset"
  | "react"
  | "all";

export interface TraverseOptions {
  files: Array<string | SymbolEntrypoint>;
  root?: string;
  tsconfig?: string;
  depth?: number;
  filters?: string[];
  targetModules?: string[];
  tests?: string[];
  relationships?: Relationship[];
  includeSymbols?: boolean;
}

export interface DependencyFile {
  path?: string;
  file?: string;
  symbol?: string;
  queueFile?: string;
  job?: string;
  module?: string;
  depth: number;
  via?: string[];
}

export interface SymbolEntrypoint {
  file: string;
  symbol?: string;
}

export interface DependencyResult {
  roots: string[];
  files: DependencyFile[];
}

export type ExportKind =
  | "function"
  | "class"
  | "const"
  | "let"
  | "var"
  | "type"
  | "interface"
  | "enum"
  | "default"
  | "re-export";

export interface SymbolsOptions {
  files: string[];
  root?: string;
  tsconfig?: string;
  config?: string;
  mode?: "list" | "signature-impact";
  symbol?: string;
  kinds?: ExportKind[];
  include?: "exports" | "imports" | "both";
}

export interface SymbolExport {
  name: string;
  kind: string;
  line: number;
  reExport?: {
    source: string;
    imported: string;
    resolved?: string;
  };
}

export interface SymbolImport {
  source: string;
  imported: string;
  local: string;
  line: number;
  typeOnly: boolean;
  resolved?: string;
}

export interface SymbolFile {
  path: string;
  exports?: SymbolExport[];
  imports?: SymbolImport[];
}

export interface SymbolsResult {
  roots: string[];
  files: SymbolFile[];
}

export interface SignatureImpactLocation {
  file: string;
  symbol: string;
  line: number;
  kind: string;
}

export interface SignatureImpactCaller {
  file: string;
  symbol?: string;
  depth: number;
  via: string[];
}

export interface SignatureImpactTest {
  file: string;
  depth: number;
  via: string[];
}

export interface SignatureImpactWarning {
  type: string;
  message: string;
}

export interface SignatureImpactResult {
  roots: string[];
  symbol: string;
  definition: SignatureImpactLocation;
  exports: SignatureImpactLocation[];
  productionCallers: SignatureImpactCaller[];
  testCallers: SignatureImpactCaller[];
  suggestedTests: SignatureImpactTest[];
  warnings: SignatureImpactWarning[];
}

export interface ProjectOptions {
  root?: string;
  tsconfig?: string;
  config?: string;
  filters?: string[];
  targets?: string[];
  files?: string[];
  roots?: string[];
  depth?: number;
  assertNoFetch?: boolean;
  direction?: "deps" | "dependents" | "both";
  /** `reactUsages` target component (`path` or `path#Symbol`). */
  target?: string;
  /** `reactUsages` `--include` spec: comma-separated `stories,tests,props`. */
  include?: string;
}

type BatchedProjectOptions = Omit<ProjectOptions, "root" | "tsconfig" | "config">;
type BatchedQueueRelatedOptions = BatchedProjectOptions & { files: string[] };
type BatchedServerRouteRelatedOptions = BatchedProjectOptions &
  ({ files: string[] } | { roots: string[] });

export interface FetchesOptions {
  root?: string;
  config?: string;
  targets?: string[];
}

export type AnalyzeProjectReportRequest =
  | ({ type: "dependencies" | "dependents" | "related"; id?: string } & Omit<
      TraverseOptions,
      "root" | "tsconfig"
    >)
  | ({ type: "symbols"; id?: string } & SymbolsOptions)
  | ({ type: "queues" | "queueEdges" | "queueCheck"; id?: string } & BatchedProjectOptions)
  | ({ type: "queueRelated"; id?: string } & BatchedQueueRelatedOptions)
  | ({
      type: "serverRoutes" | "serverRouteList" | "serverRouteEdges";
      id?: string;
    } & BatchedProjectOptions)
  | ({ type: "serverRouteRelated"; id?: string } & BatchedServerRouteRelatedOptions)
  | ({ type: "reactAnalyze" | "reactCheck"; id?: string } & Pick<
      ProjectOptions,
      "targets" | "depth" | "assertNoFetch"
    >)
  | ({
      type: "playwrightCheck" | "playwrightEdges" | "playwrightTests";
      id?: string;
    } & Omit<PlaywrightOptions, "root" | "config">)
  | ({ type: "playwrightRelated"; id?: string } & Omit<PlaywrightRelatedOptions, "root" | "config">)
  | { type: "check"; id?: string };

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
    result: unknown;
  }>;
}
