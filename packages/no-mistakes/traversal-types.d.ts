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
  | "http"
  | "process"
  | "asset"
  | "react"
  | "dotnet"
  | "swift"
  | "terraform"
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
