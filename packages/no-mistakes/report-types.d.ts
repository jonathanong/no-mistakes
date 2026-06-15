export interface PlaywrightOptions {
  root?: string;
  config?: string;
  playwrightConfig?: string[];
  project?: string;
  files?: string[];
  assertConditionalTests?: boolean;
  allowSkippedTests?: boolean;
  assertUniqueTestIds?: boolean;
  assertUniqueHtmlIds?: boolean;
}

export interface PlaywrightRelatedOptions extends PlaywrightOptions {
  files: string[];
}

export interface CheckReport {
  react: ReactViolation[];
  queues: unknown[];
  rules: unknown[];
  integration: unknown[];
  codebase: unknown[];
  warnings: string[];
  advisories: unknown[];
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

export type FetchSourceType =
  | "page"
  | "layout"
  | "loading"
  | "error"
  | "template"
  | "route"
  | "module";

export type CacheKind =
  | "none"
  | "fetch-cache"
  | "fetch-next-revalidate"
  | "fetch-next-tags"
  | "react-cache"
  | "cache"
  | "unstable-cache";

export interface FetchOccurrence {
  path: string;
  rawPath: string;
  method: string;
  file: string;
  line: number;
  side: "server" | "client";
  rsc: boolean;
  cached: boolean;
  cacheKind: CacheKind;
  cachedFunction?: string;
  dynamic: boolean;
  unsupported: boolean;
  functionName?: string;
  conditional: boolean;
  inPromiseAll: boolean;
  errorHandled: boolean;
  sourceType: FetchSourceType;
}

export interface FetchRouteReport {
  route: string;
  file: string;
  apiCalls: FetchOccurrence[];
}

export interface FetchSummary {
  totalRoutes: number;
  routesWithApiCalls: number;
  totalApiCalls: number;
  uniqueApiCalls: number;
  duplicateApiCalls: number;
  dynamicApiCalls: number;
  cachedApiCalls: number;
  clientApiCalls: number;
  serverApiCalls: number;
  rscApiCalls: number;
  conditionalApiCalls: number;
  parallelApiCalls: number;
  errorHandledApiCalls: number;
}

export interface FetchReport {
  summary: FetchSummary;
  routes: FetchRouteReport[];
  duplicates: unknown[];
  unsupported: unknown[];
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

export interface ReactCallsite {
  file: string;
  line: number;
  component: string;
  props: string[];
  hasSpread: boolean;
}

export interface ReactUsagesReport {
  target: { file: string; symbol?: string };
  callsites: ReactCallsite[];
  /** Story files importing the target. Omitted when `props`/`tests`-only `include`. */
  stories?: string[];
  /** Test files importing the target. */
  tests?: string[];
  /** Exported prop type/interface names declared in the target file. */
  propTypes?: string[];
}

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
