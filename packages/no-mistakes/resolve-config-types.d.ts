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
  coverageRoutes: boolean;
  coverageSelectors: boolean;
  frontendRoot?: string | null;
  selectorRoots: string[];
  apps: ResolvedPlaywrightApp[];
}

export interface ResolvedPlaywrightApp {
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
