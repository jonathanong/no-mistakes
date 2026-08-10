import type {
  CheckReport,
  FetchReport,
  QueueReport,
  ReactComponentFacts,
} from "../../../packages/no-mistakes/index";

// Keep representative values assignable to the declarations so JSON casing
// and nullable/omitted fields cannot drift independently of the public DTOs.
const fetchReport: FetchReport = {
  summary: {
    totalRoutes: 0,
    routesWithApiCalls: 0,
    totalApiCalls: 0,
    uniqueApiCalls: 0,
    duplicateApiCalls: 0,
    dynamicApiCalls: 0,
    cachedApiCalls: 0,
    clientApiCalls: 0,
    serverApiCalls: 0,
    rscApiCalls: 0,
    conditionalApiCalls: 0,
    parallelApiCalls: 0,
    errorHandledApiCalls: 0,
  },
  routes: [],
  duplicates: [],
  unsupported: [],
};
const queueReport: QueueReport = {
  producers: [],
  workers: [],
  jobs: [],
  edges: [],
  diagnostics: [],
  check: [],
};
const componentFacts: ReactComponentFacts[] = [];
const checkReport: CheckReport = {
  react: [{
    component: "Example",
    file: "src/example.ts",
    rule: "example-rule",
    detail: null,
  }],
  queues: [],
  rules: [],
  integration: [],
  codebase: [],
  warnings: [],
  advisories: [],
};

void fetchReport;
void queueReport;
void componentFacts;
void checkReport;
