export type ImportUsageKind = "static" | "type" | "dynamic" | "require" | "require-resolve";

export interface ImportUsage {
  specifier: string;
  packageName: string | null;
  kind: ImportUsageKind;
  line: number;
  sideEffectOnly: boolean;
  reExport: boolean;
}

export interface ImportUsageFile {
  path: string;
  imports: ImportUsage[];
}

export interface ImportUsagesOptions {
  files?: string[];
  /** Project root. Defaults to the current working directory. */
  root?: string;
  scanRoots?: string[];
  filters?: string[];
}

export interface ImportUsagesResult {
  roots: string[];
  files: ImportUsageFile[];
}
