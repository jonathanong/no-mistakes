const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { pathToFileURL } = require("node:url");
const test = globalThis.test || require("node:test").test;

const packageRoot = join(__dirname, "..");
const repositoryRoot = join(packageRoot, "..", "..");
const fixture = (name) => join(repositoryRoot, "test-cases", "tests-audit", name);
const addonPath = join(repositoryRoot, "fixtures", "napi", "test-addon.js");
const indexPath = join(packageRoot, "index.js");
const planningPath = join(packageRoot, "planning.js");

async function withApi(callback) {
  const previous = globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__;
  const calls = [];
  globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__ = {
    testsAuditJson: async (input) => {
      const request = JSON.parse(input);
      calls.push(request);
      return JSON.stringify({ missed_observed_tests: [{ test_file: "missed.test.mts" }] });
    },
    analyzeProjectJson: async (input) => {
      const request = JSON.parse(input);
      calls.push(request);
      return JSON.stringify({
        reports: [
          {
            type: "testsAudit",
            result: {
              selected_without_observed_execution: [{ test_file: "excess.test.mts" }],
            },
          },
        ],
      });
    },
  };
  for (const file of [indexPath, planningPath, addonPath])
    delete require.cache[require.resolve(file)];
  try {
    await callback(require(indexPath), calls);
  } finally {
    globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__ = previous;
    for (const file of [indexPath, planningPath, addonPath])
      delete require.cache[require.resolve(file)];
  }
}

test("testsAudit accepts saved artifacts, returns a promise and camelizes execution evidence", async () => {
  await withApi(async (api, calls) => {
    const promise = api.testsAudit({
      plan: fixture("plan.json"),
      observations: fixture("observations.json"),
    });
    assert.equal(typeof promise.then, "function");
    assert.deepEqual(await promise, { missedObservedTests: [{ testFile: "missed.test.mts" }] });
    // Saved traces must reach the worker as paths, without JS file decoding or cloning.
    assert.equal(calls[0].plan, fixture("plan.json"));
    assert.equal(calls[0].observations, fixture("observations.json"));
    assert.equal(calls[0].planJson, undefined);
    assert.equal(calls[0].observationsJson, undefined);
    const esm = await import(`${pathToFileURL(join(packageRoot, "index.mjs"))}?audit-export`);
    assert.equal(typeof esm.testsAudit, "function");
  });
});

test("testsAudit leaves artifact objects and JSON text for the native worker and converts results", async () => {
  await withApi(async (api, calls) => {
    const camel = (value) => require(planningPath).camelizeValue(value);
    const plan = camel(JSON.parse(readFileSync(fixture("plan.json"), "utf8")));
    const observations = camel(JSON.parse(readFileSync(fixture("observations.json"), "utf8")));
    const text = JSON.stringify(observations, null, 2);
    await api.testsAudit({ planJson: plan, observationsJson: text });
    assert.deepEqual(calls[0].planJson, plan);
    // Preserving whitespace/text proves the facade did not parse and normalize the trace.
    assert.equal(calls[0].observationsJson, text);
    const aggregate = await api.analyzeProject({
      reports: [{ type: "testsAudit", planJson: plan, observations: fixture("observations.json") }],
    });
    assert.equal(
      calls[1].reports[0].planJson.plan.selectedTests[0].testFile,
      "tests/selected.test.mts",
    );
    assert.equal(calls[1].reports[0].observations, fixture("observations.json"));
    assert.equal(
      aggregate.reports[0].result.selectedWithoutObservedExecution[0].testFile,
      "excess.test.mts",
    );
  });
});

test("testsAudit preserves native validation for unreadable malformed and conflicting inputs", async () => {
  await withApi(async (api, calls) => {
    await api.testsAudit({ plan: fixture("invalid.json"), observations: fixture("missing.json") });
    assert.equal(calls[0].plan, fixture("invalid.json"));
    assert.equal(calls[0].observations, fixture("missing.json"));
    await api.testsAudit({ planJson: "invalid", observationsJson: "invalid" });
    assert.equal(calls[1].planJson, "invalid");
    assert.equal(calls[1].observationsJson, "invalid");
    await api.testsAudit({
      plan: "conflict.json",
      planJson: {},
      observations: "conflict.json",
      observationsJson: {},
    });
    assert.equal(calls[2].plan, "conflict.json");
    assert.deepEqual(calls[2].planJson, {});
    assert.equal(calls[2].observations, "conflict.json");
    assert.deepEqual(calls[2].observationsJson, {});
    await api.testsAudit();
    assert.deepEqual(calls[3], {});
  });
});
