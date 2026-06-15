// Lightweight single-file query commands (issue #417).

export interface QueryFileOptions {
  /** The TS/JS file to query (relative to `root` or absolute). */
  file: string;
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to tsconfig.json for alias resolution. Searched upward if omitted. */
  tsconfig?: string;
}

export interface ImportersOptions extends QueryFileOptions {
  /** Also compute the transitive impacted-test set (builds the dependency graph). */
  tests?: boolean;
}

export interface ImportersTestImpact {
  tests: string[];
  count: number;
}

export interface ImportersResult {
  file: string;
  directImporters: string[];
  dependentsCount: number;
  testImpact?: ImportersTestImpact;
}

export interface ExportsOfOptions extends QueryFileOptions {
  /** Skip the reverse import scan; only list exports. */
  noImporters?: boolean;
}

export interface ExportRowResult {
  name: string;
  kind: string;
  line: number;
  /** Resolved re-export target, root-relative. Only present for re-exports. */
  resolved?: string;
  importers: string[];
}

export interface ExportsOfResult {
  file: string;
  exports: ExportRowResult[];
}

export interface DeadExportsOptions extends QueryFileOptions {
  /** Specific export names to check. Defaults to every export of the file. */
  names?: string[];
}

export interface DeadExportResult {
  name: string;
  referenced: boolean;
  importerCount: number;
}

export interface DeadExportsResult {
  file: string;
  results: DeadExportResult[];
  anyDead: boolean;
}

export interface CallSitesOptions extends QueryFileOptions {
  /** The exported function name to find call sites for. */
  exportName: string;
}

export type ArgShape =
  | "string"
  | "number"
  | "boolean"
  | "null"
  | "identifier"
  | "object"
  | "array"
  | "arrow"
  | "call"
  | "spread"
  | "other";

export interface CallSite {
  file: string;
  line: number;
  /** Enclosing named function, when determinable. */
  caller?: string;
  argCount: number;
  hasSpread: boolean;
  args: ArgShape[];
}

export interface CallSitesResult {
  file: string;
  export: string;
  callSites: CallSite[];
}

export type ResolveCheckOptions = QueryFileOptions;

export type ImportResolutionStatus = "resolved" | "unresolved" | "external";

export interface ResolveCheckImport {
  specifier: string;
  kind: "static" | "type" | "dynamic" | "require";
  status: ImportResolutionStatus;
  /** Root-relative resolved target, when the import resolves locally. */
  resolved?: string;
}

export interface ResolveCheckResult {
  file: string;
  allResolve: boolean;
  imports: ResolveCheckImport[];
  unresolved: string[];
}
