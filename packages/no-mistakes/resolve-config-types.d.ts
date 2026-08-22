export interface ResolvedConfig {
  configPath?: string | null;
  frontendApps: ResolvedFrontendApp[];
  playwright: ResolvedPlaywright;
  vitestFullSuiteTriggers: ResolvedTrigger[];
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
}

export interface ResolvedTrigger {
  name: string;
  paths: string[];
  targets: string[];
  source: "triggers" | "projects";
}
