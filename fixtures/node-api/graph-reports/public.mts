import {
  playwrightCheck,
  playwrightEdges,
  playwrightRelated,
  playwrightTests,
  serverRouteList,
  serverRoutes,
} from "../../../packages/no-mistakes/index.js";
import type {
  PlaywrightCoverageReport,
  PlaywrightEdgesReport,
  PlaywrightRelatedReport,
  PlaywrightTestsReport,
  ServerRoute,
} from "../../../packages/no-mistakes/index.js";

const coverage: PlaywrightCoverageReport = await playwrightCheck({ root: "." });
const totalRoutes: number = coverage.summary.totalRoutes;
const covered: boolean | undefined = coverage.selectors[0]?.covered;
const duplicateLine: number | undefined = coverage.duplicateSelectors[0]?.line;
const fetchPath: string | undefined = coverage.fetchApis[0]?.path;
const routeTests: string[] | undefined = coverage.routes[0]?.tests;

const edges: PlaywrightEdgesReport = await playwrightEdges({ root: "." });
for (const edge of edges.edges) {
  switch (edge.kind) {
    case "fetch":
      edge.method.toUpperCase();
      break;
    case "route":
      edge.url.toUpperCase();
      break;
    case "selector":
      edge.selector.toUpperCase();
      break;
    case "locatorText":
      edge.selectorRefs[0]?.attribute.toUpperCase();
      break;
    default: {
      const exhaustive: never = edge;
      void exhaustive;
    }
  }
}

const related: PlaywrightRelatedReport = await playwrightRelated({ files: ["app/page.tsx"] });
const relatedTests: string[] = related.tests;
const relatedFetchApis: string[] = related.fetch_apis;

const inventory: PlaywrightTestsReport = await playwrightTests({ root: "." });
const testIds: string[] | undefined = inventory.tests[0]?.testIds;
const routes: string[] | undefined = inventory.tests[0]?.routes;

const serverRows: ServerRoute[] = await serverRouteList({ root: "." });
const route: string | undefined = serverRows[0]?.route;
const serverRoutesReport: ServerRoute[] = (await serverRoutes({ root: "." })).routes;

void [
  totalRoutes,
  covered,
  duplicateLine,
  fetchPath,
  routeTests,
  relatedTests,
  relatedFetchApis,
  testIds,
  routes,
  route,
  serverRoutesReport,
];
