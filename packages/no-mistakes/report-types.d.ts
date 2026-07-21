export interface PlaywrightOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
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

export interface InfraOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  /** `infraResourceRefs` address (`<type>.<name>`). */
  address?: string;
  /** `infraOutputs` module directory (relative to root). */
  moduleDir?: string;
  /** `infraTestFor` `.tf` file (relative to root). */
  tfFile?: string;
}

export interface ResourceRefRow {
  /** The referencing block's address. */
  address: string;
  /** The referencing file, relative to the root. */
  file: string;
}

export interface ModuleOutput {
  name: string;
  references: string[];
}

export interface OutputConsumer {
  output: string;
  from: string;
  file: string;
}

export interface ModuleOutputsResult {
  module: string;
  exports: ModuleOutput[];
  consumers: OutputConsumer[];
}

export interface TestForRow {
  test_file: string;
}

export interface SwiftOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  /** The Swift source file to query (relative to root). */
  file?: string;
}

export interface SwiftImporterRow {
  file: string;
  depth: number;
}

export interface SwiftTestTargetRow {
  target: string;
  package: string;
  command: string;
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
