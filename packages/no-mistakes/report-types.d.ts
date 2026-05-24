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
  assertUniqueSelectors?: boolean;
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
