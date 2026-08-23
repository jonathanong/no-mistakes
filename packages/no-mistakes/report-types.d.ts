export * from "./check-report-types";
export * from "./fetch-report-types";
export * from "./queue-report-types";
export * from "./react-report-types";

export interface PlaywrightOptions {
  /** Project root. Defaults to the current working directory. */
  root?: string;
  /** Path to the no-mistakes config file (e.g. .no-mistakes.yml). Auto-discovered in root if omitted. */
  config?: string;
  playwrightConfig?: string[];
  project?: string;
  /**
   * The `.no-mistakes.yml` `projects:` key of the frontend app to analyze.
   * Only needed when the repository configures more than one `type: nextjs`
   * project and `tests.playwright.apps.<project>.project` is not set.
   */
  app?: string;
  files?: string[];
  assertConditionalTests?: boolean;
  allowSkippedTests?: boolean;
  assertUniqueTestIds?: boolean;
  assertUniqueHtmlIds?: boolean;
}

export interface PlaywrightRelatedOptions extends PlaywrightOptions {
  files: string[];
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
