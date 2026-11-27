import type { RouteCoverageFramework } from "./resolve-config-types";

export interface PlaywrightCoverageSummary {
  totalRoutes: number;
  coveredRoutes: number;
  uncoveredRoutes: number;
  totalSelectors: number;
  coveredSelectors: number;
  uncoveredSelectors: number;
  duplicateSelectors: number;
  totalFetchApis: number;
  coveredFetchApis: number;
  uncoveredFetchApis: number;
}

export interface PlaywrightRouteCoverageAttribution {
  framework: RouteCoverageFramework;
  project: string;
  declarationFile: string;
}

export interface PlaywrightTestReference {
  file: string;
  name?: string;
  describePath?: string[];
  attribution?: PlaywrightRouteCoverageAttribution;
}

export interface PlaywrightCoverageRoute {
  route: string;
  file: string;
  covered: boolean;
  tests: string[];
  testsDetail: PlaywrightTestReference[];
  urls: string[];
}

export interface PlaywrightSelectorHelperReference {
  testFile: string;
  line: number;
  call: string;
}

export interface PlaywrightCoverageSelector {
  attribute: string;
  value: string;
  file: string;
  covered: boolean;
  unsupportedDynamic: boolean;
  tests: string[];
  testsDetail: PlaywrightTestReference[];
  selectors: string[];
  helperReferences?: PlaywrightSelectorHelperReference[];
}

export interface PlaywrightCoverageFetch {
  method: string;
  path: string;
  covered: boolean;
  tests: string[];
  testsDetail: PlaywrightTestReference[];
  routeFiles: string[];
}

export interface PlaywrightDuplicateSelector {
  attribute: string;
  value: string;
  file: string;
  line: number;
}

export interface PlaywrightEdgeTest {
  testFile: string;
  testName?: string;
  describePath?: string[];
}

export interface PlaywrightFetchEdge extends PlaywrightEdgeTest {
  kind: "fetch";
  routeFile: string;
  route: string;
  method: string;
  path: string;
  side: "client" | "server";
  cached: boolean;
}

export interface PlaywrightRouteEdge extends PlaywrightEdgeTest {
  kind: "route";
  routeFile: string;
  route: string;
  url: string;
  hook: boolean;
  line: number;
  attribution?: PlaywrightRouteCoverageAttribution;
}

export interface PlaywrightSelectorEdge extends PlaywrightEdgeTest {
  kind: "selector";
  appFile: string;
  attribute: string;
  value: string;
  selector: string;
  line: number;
}

export interface PlaywrightSelectorReference {
  attribute: string;
  value: string;
}

export interface PlaywrightLocatorTextEdge extends PlaywrightEdgeTest {
  kind: "locatorText";
  appFile: string;
  locatorKind: string;
  role?: string;
  text: string;
  locator: string;
  selectorRefs: PlaywrightSelectorReference[];
  reasons: string[];
  line: number;
}

export type PlaywrightEdge =
  | PlaywrightFetchEdge
  | PlaywrightRouteEdge
  | PlaywrightSelectorEdge
  | PlaywrightLocatorTextEdge;

export interface PlaywrightEdgesReport {
  edges: PlaywrightEdge[];
}

export interface PlaywrightRelatedReport {
  tests: string[];
  /** Fetch identities, using the same field spelling as the CLI JSON. */
  fetch_apis: string[];
}

export interface PlaywrightTestEntry {
  file: string;
  name?: string;
  describePath?: string[];
  testIds: string[];
  htmlIds: string[];
  routes: string[];
  fetchApis: string[];
  locatorTexts: string[];
}

export interface PlaywrightTestsReport {
  tests: PlaywrightTestEntry[];
}
