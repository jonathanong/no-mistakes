import type { PlaywrightOptions } from "./report-types";

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
  files: string[];
  root?: string;
  tsconfig?: string;
  depth?: number;
  filters?: string[];
  targetModules?: string[];
  tests?: string[];
  relationships?: Relationship[];
}

export interface DependencyFile {
  path?: string;
  queueFile?: string;
  job?: string;
  module?: string;
  depth: number;
  via?: string[];
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
}

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
  | ({ type: "symbols"; id?: string } & Omit<SymbolsOptions, "root" | "tsconfig">)
  | ({ type: "queues" | "queueEdges" | "queueRelated" | "queueCheck"; id?: string } & Omit<
      ProjectOptions,
      "root" | "tsconfig" | "config"
    >)
  | ({
      type: "serverRoutes" | "serverRouteList" | "serverRouteEdges" | "serverRouteRelated";
      id?: string;
    } & Omit<ProjectOptions, "root" | "tsconfig" | "config">)
  | ({ type: "reactAnalyze" | "reactCheck"; id?: string } & Pick<
      ProjectOptions,
      "targets" | "depth" | "assertNoFetch"
    >)
  | ({
      type: "playwrightCheck" | "playwrightEdges" | "playwrightRelated" | "playwrightTests";
      id?: string;
    } & Omit<PlaywrightOptions, "root" | "config">)
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
