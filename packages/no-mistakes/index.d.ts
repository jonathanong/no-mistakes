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

export interface QueueReport {
  producers: unknown[];
  workers: unknown[];
  jobs: unknown[];
  edges: unknown[];
  diagnostics: unknown[];
  check: unknown[];
}

export interface GraphEdge {
  from: string;
  to: string;
  kind: string;
}

export interface ServerRoutesReport {
  summary: {
    totalRoutes: number;
    totalFiles: number;
    dynamicRoutes: number;
  };
  routes: unknown[];
  edges: unknown[];
  diagnostics: unknown[];
}

export interface ReactComponentFacts {
  name: string;
  file: string;
  environment: "server" | "client" | "shared" | "unknown";
  hasState: boolean;
  hasProps: boolean;
  passesProps: boolean;
  usesMemo: boolean;
  usesContextProvider: boolean;
  usesSuspense: boolean;
  fetches: unknown[];
  dependencies: string[];
  children: unknown[];
  inheritedFromChildren?: unknown;
}

export interface ReactViolation {
  component: string;
  file: string;
  rule: string;
  detail?: string;
}

export function dependencies(options: TraverseOptions): Promise<DependencyResult>;
export function dependents(options: TraverseOptions): Promise<DependencyResult>;
export function related(options: TraverseOptions): Promise<DependencyResult>;
export function symbols(options: SymbolsOptions): Promise<SymbolsResult>;
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
export function version(): Promise<string>;
