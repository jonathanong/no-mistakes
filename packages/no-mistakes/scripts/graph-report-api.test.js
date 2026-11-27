const assert = require("node:assert/strict");
const { execFileSync } = require("node:child_process");
const { readFileSync } = require("node:fs");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const compiled = addon?.endsWith(".node");
const repository = join(__dirname, "../../..");
const fixture = (...parts) => join(repository, "test-cases", ...parts, "fixture");
const snapshot = JSON.parse(
  readFileSync(join(__dirname, "snapshots/graph-report-api.snap"), "utf8"),
);

test(
  "compiled Playwright methods return the declared graph report shapes",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const root = fixture("nextjs-coverage", "with-fetches");
    const coverage = await api.playwrightCheck({ root });
    const { edges } = await api.playwrightEdges({ root });
    const related = await api.playwrightRelated({ root, files: ["app/page.tsx"] });
    const inventory = await api.playwrightTests({ root });
    assert.deepEqual(
      {
        summary: coverage.summary,
        route: coverage.routes[0],
        fetch: coverage.fetchApis[0],
        fetchEdge: edges.find((edge) => edge.kind === "fetch"),
        routeEdge: edges.find((edge) => edge.kind === "route" && edge.describePath?.length),
        related,
        test: inventory.tests.find((entry) => entry.describePath?.length),
      },
      snapshot.playwright,
    );
  },
);

test(
  "compiled server route list and report expose the same route rows",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const root = fixture("codebase-analysis", "routes/good");
    const rows = await api.serverRouteList({ root });
    assert.deepEqual(rows, snapshot.serverRoutes);
    assert.deepEqual((await api.serverRoutes({ root })).routes, snapshot.serverRoutes);
  },
);

test(
  "compiled Playwright check applies conditional and skipped test flags",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const root = fixture("nextjs-coverage", "skipped-tests");
    const defaults = await api.playwrightCheck({ root });
    const assertConditional = await api.playwrightCheck({ root, assertConditionalTests: true });
    const allowSkipped = await api.playwrightCheck({ root, allowSkippedTests: true });
    const coverage = (report) =>
      Object.fromEntries(report.routes.map(({ route, covered }) => [route, covered]));
    assert.deepEqual(
      [defaults, assertConditional, allowSkipped].map(coverage),
      snapshot.skipPolicies,
    );
  },
);

test("compiled Playwright check selects the same app as the CLI", { skip: !compiled }, async () => {
  const api = require("../index.js");
  const root = fixture("codebase-analysis", "graph-multi-frontend-apps");
  const app = "control-web";
  const cli =
    process.env.NO_MISTAKES_TEST_CLI_PATH ||
    join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
  const report = await api.playwrightCheck({ root, app });
  const cliReport = JSON.parse(
    execFileSync(cli, ["playwright", "check", "--root", root, "--app", app, "--json"], {
      encoding: "utf8",
    }),
  );
  assert.deepEqual(report.routes, cliReport.routes);
  assert.deepEqual(report.routes, snapshot.appRoutes);
  assert.deepEqual(report.selectors, snapshot.appSelectors);
});

test(
  "compiled dataPw honors the attribute, scan root, and section options",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const options = { root: fixture("codebase-analysis", "data-pw"), value: "search-bar" };
    assert.deepEqual(
      {
        attributes: await api.dataPw({ ...options, attributes: ["data-testid"] }),
        scan: await api.dataPw({ ...options, scan: ["other"] }),
        testOnly: await api.dataPw({ ...options, include: "test" }),
      },
      snapshot.dataPw,
    );
  },
);
