export interface DataPwOptions {
  root?: string;
  config?: string;
  /** The selector-attribute value to find (e.g. `search-bar`). */
  value: string;
  /** Attribute names to scan instead of the configured `testIds`. */
  attributes?: string[];
  /** Source path prefixes to scan instead of the configured `selectorRoots`. */
  scan?: string[];
  /** Comma-separated subset of `source,test` (default: both). */
  include?: string;
}

export interface DataPwHit {
  file: string;
  line: number;
  attribute: string;
}

export interface DataPwReport {
  value: string;
  attributes: string[];
  /** Source-file matches. Omitted when `--include test`. */
  source?: DataPwHit[];
  /** Test-file matches. Omitted when `--include source`. */
  test?: DataPwHit[];
}

export interface EffectsOptions {
  root?: string;
  tsconfig?: string;
  config?: string;
  /** Effect kind to resolve (a key under `effects:` in config). */
  kind: string;
  /** Entry file whose transitive imports are scanned. */
  entry: string;
  /** Restrict to one or more configured categories. */
  categories?: string[];
  /** Maximum traversal depth (default: unlimited). */
  depth?: number;
}

export interface EffectCallSite {
  file: string;
  line: number;
  callee: string;
  category?: string;
  caller?: string;
  depth: number;
}

export interface EffectsReport {
  kind: string;
  entry: string;
  callSites: EffectCallSite[];
  byCategory: Record<string, number>;
}

export interface RscCallersOptions {
  root?: string;
  tsconfig?: string;
  config?: string;
  /** Component file to find RSC callers of. */
  component: string;
  /** Maximum traversal depth (default: unlimited). */
  depth?: number;
}

export interface RscCaller {
  file: string;
  kind: "page" | "component";
  environment: "server" | "client" | "unknown";
  depth: number;
}

export interface RscCallersReport {
  component: string;
  callers: RscCaller[];
}

export interface RegistryExtensionOptions {
  root?: string;
  /** Registry file to summarize the entry pattern of. */
  registryFile: string;
}

export interface RegistryEntryImport {
  specifier: string;
  symbol?: string;
  local: string;
  kind: string;
}

export interface RegistryEntry {
  line: number;
  import?: RegistryEntryImport;
  callShape: string;
}

export interface RegistryExtensionReport {
  registryFile: string;
  patternKind: string;
  registrant?: string;
  confidence: string;
  entries: RegistryEntry[];
  template?: string;
  notes: string[];
}
