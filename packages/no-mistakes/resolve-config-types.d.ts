import type { ProjectOptions } from "./traversal-types";
import type { TestPlanFramework } from "./test-types";

export interface ResolvedConfig {
  configPath?: string | null;
  frontendApps: ResolvedFrontendApp[];
  playwright: ResolvedPlaywright;
  vitestFullSuiteTriggers: ResolvedTrigger[];
  fullSuiteTriggers: ResolvedFrameworkTriggers[];
}

export interface ResolvedFrontendApp {
  project?: string | null;
  root: string;
  routeRoot: string;
  selectorRoots: string[];
}

export interface ResolvedPlaywright {
  routeCoverageSources?: RouteCoverageSource[];
  coverageRoutes: boolean;
  coverageSelectors: boolean;
  frontendRoot?: string | null;
  selectorRoots: string[];
  apps: ResolvedPlaywrightApp[];
}

export interface ResolvedPlaywrightApp {
  routeCoverageSources?: RouteCoverageSource[];
  playwrightProject: string;
  project?: string | null;
  frontendRoot?: string | null;
  selectorRoots: string[];
  rewrites: ResolvedRewrite[];
  ignoreRoutes: string[];
}

export interface ResolvedRewrite {
  source: string;
  destination: string;
}

export type RouteCoverageFramework = "vitest";

export interface RouteCoverageHelper {
  /** Exact repository-relative helper module or import-resolvable specifier. */
  module: string;
  export: string;
  /** An imported class instance method; omit for an imported function. */
  method?: string | null;
  urlArgument: number;
}

export interface RouteCoverageSource {
  framework: RouteCoverageFramework;
  project: string;
  include: string[];
  /** Exact canonical page route identifiers, optionally with named parameters. */
  routes: string[];
  helpers: RouteCoverageHelper[];
}

export interface RouteCoverageAttribution {
  framework: RouteCoverageFramework;
  project: string;
  declarationFile: string;
}

export interface ResolvedTrigger {
  name: string;
  paths: string[];
  targets: string[];
  /** Effective changed-test expansion policy; present only for structured triggers. */
  includeChangedTests?: boolean;
  source: "triggers" | "projects";
}

export interface ResolvedFrameworkTriggers {
  framework: TestPlanFramework;
  triggers: ResolvedTrigger[];
}

/** Select the existing resolved report or the source-level merged declarations. */
export type ConfigResolveView = "resolved" | "declared";
export interface ConfigResolveOptions extends ProjectOptions {
  view?: ConfigResolveView;
}

/** JSON-compatible source values; omitted properties remain omitted. */
export type DeclaredConfigValue =
  | null
  | boolean
  | number
  | string
  | DeclaredConfigValue[]
  | DeclaredConfigObject;
export interface DeclaredConfigObject {
  [key: string]: DeclaredConfigValue | undefined;
}
export interface DeclaredRule extends DeclaredConfigObject {
  rule?: string;
  name?: string | null;
  options?: DeclaredConfigValue;
}
/** Repository policy for proven stale configured exception entries. */
export type UnusedAllowEntryPolicy = "advise" | "report" | "ignore";

/** Canonical source-level merge, before defaults, expansion, or materialization. */
export interface DeclaredConfig extends DeclaredConfigObject {
  /** Defaults to `advise`; absent literal file paths also prove staleness. */
  reportUnusedAllowEntries?: UnusedAllowEntryPolicy;
  rules?: DeclaredRule[];
}
