export type Relationship =
  | "import"
  | "import-static"
  | "import-dynamic"
  | "import-type"
  | "import-require"
  | "route-import"
  | "workspace"
  | "package"
  | "test"
  | "route"
  | "queue"
  | "md"
  | "ci"
  | "workflow"
  | "workflow-job"
  | "workflow-step"
  | "workflow-needs"
  | "workflow-uses"
  | "workflow-run"
  | "workflow-artifact"
  | "http"
  | "process"
  | "asset"
  | "react"
  | "dotnet"
  | "swift"
  | "terraform"
  | "resource"
  | "all";

export interface TraverseOptions {
  files: Array<string | SymbolEntrypoint>;
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to tsconfig.json for alias resolution. Searched upward if omitted. */
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
  /** Workflow file for a virtual GitHub Actions job or step node. */
  workflowFile?: string;
  /** GitHub Actions job identifier for a virtual workflow job or step node. */
  job?: string;
  /** Zero-based step index for a virtual GitHub Actions workflow step node. */
  step?: number;
  module?: string;
  depth: number;
  via?: string[];
}

export interface SymbolEntrypoint {
  file: string;
  symbol?: string;
}

export interface TsConfigDiagnostic {
  kind: "ambiguous-ownership" | "invalid-config" | "invalid-extends" | "invalid-reference";
  config: string | null;
  file: string | null;
  detail: string;
  candidates: string[];
}

export interface TsConfigProvenance {
  importer: string;
  config: string | null;
  forced: boolean;
}

export interface DependencyResult {
  roots: string[];
  files: DependencyFile[];
  diagnostics: TsConfigDiagnostic[];
  tsconfig_provenance: TsConfigProvenance[];
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
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to tsconfig.json for alias resolution. Searched upward if omitted. */
  tsconfig?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  mode?: "list" | "signature-impact";
  symbol?: string;
  kinds?: ExportKind[];
  include?: "exports" | "imports" | "both";
}

export type SymbolsListOptions = Omit<SymbolsOptions, "mode" | "symbol"> & {
  mode?: "list";
  symbol?: string;
};

export type SymbolsSignatureImpactOptions = SymbolsOptions & {
  mode: "signature-impact";
  symbol: string;
};

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
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to tsconfig.json for alias resolution. Searched upward if omitted. */
  tsconfig?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
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
