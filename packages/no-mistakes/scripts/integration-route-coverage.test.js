const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const addonPath = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const root = join(
  __dirname,
  "..",
  "..",
  "..",
  "fixtures",
  "playwright",
  "integration-route-coverage",
);

test(
  "compiled async APIs preserve integration route ownership and scoped configuration",
  {
    skip: !addonPath?.endsWith(".node"),
    timeout: 20_000,
  },
  async () => {
    const api = require("../index.js");
    const pending = api.resolveConfig({ root });
    assert.equal(typeof pending.then, "function");
    const config = await pending;
    assert.equal(config.playwright.routeCoverageSources[0].project, "integration");
    assert.deepEqual(config.playwright.routeCoverageSources[0].helpers[1], {
      module: "integration/client.ts",
      export: "loadPage",
      urlArgument: 1,
    });
    const report = await api.playwrightCheck({ root });
    const health = report.routes.find((route) => route.route === "/healthz");
    assert.equal(health.covered, true);
    assert.deepEqual(health.tests, ["integration/web.test.ts"]);
    assert.deepEqual(health.testsDetail[0].attribution, {
      framework: "vitest",
      project: "integration",
      declarationFile: "integration/cases.ts",
    });
    assert.equal(report.routes.find((route) => route.route === "/unregistered").covered, false);
    const edges = await api.playwrightEdges({ root });
    assert(
      edges.edges.some(
        (edge) =>
          edge.kind === "route" &&
          edge.route === "/healthz" &&
          edge.testFile === "integration/web.test.ts" &&
          edge.attribution?.framework === "vitest",
      ),
    );
    assert(!edges.edges.some((edge) => edge.kind !== "route" && edge.attribution));
  },
);
