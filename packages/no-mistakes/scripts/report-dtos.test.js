const assert = require("node:assert/strict");
const test = globalThis.test || require("node:test").test;
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const packageRoot = join(__dirname, "..");
const repositoryRoot = join(packageRoot, "..", "..");
const addonPath = join(packageRoot, "bin", "no-mistakes.node");
const indexPath = join(packageRoot, "index.js");
const planningPath = join(packageRoot, "planning.js");
const fixture = JSON.parse(
  readFileSync(
    join(repositoryRoot, "fixtures", "node-api", "report-dtos", "reports.json"),
    "utf8",
  ),
);

function loadApiWithFixtureNative() {
  delete require.cache[require.resolve(indexPath)];
  delete require.cache[require.resolve(planningPath)];
  delete require.cache[addonPath];

  const previous = require.extensions[".node"];
  require.extensions[".node"] = (module, filename) => {
    assert.equal(filename, addonPath);
    module.exports = {
      fetchesJson: async () => JSON.stringify(fixture.fetches),
      checkJson: async () => JSON.stringify(fixture.check),
      queuesJson: async () => JSON.stringify(fixture.queues),
      reactAnalyzeJson: async () => JSON.stringify(fixture.reactAnalyze),
    };
  };

  return {
    api: require(indexPath),
    restore() {
      require.extensions[".node"] = previous;
      delete require.cache[require.resolve(indexPath)];
      delete require.cache[require.resolve(planningPath)];
      delete require.cache[addonPath];
    },
  };
}

test("Node report DTO fixture preserves fetch, queue, React, and check shapes", async () => {
  const loaded = loadApiWithFixtureNative();
  try {
    assert.deepEqual(await loaded.api.fetches({ root: "/fixture" }), fixture.fetches);
    assert.deepEqual(await loaded.api.queues({ root: "/fixture" }), fixture.queues);
    assert.deepEqual(await loaded.api.reactAnalyze({ root: "/fixture" }), fixture.reactAnalyze);
    assert.deepEqual(await loaded.api.check({ root: "/fixture" }), fixture.check);
  } finally {
    loaded.restore();
  }
});

test("Node report declarations name every fixture collection precisely", () => {
  const declarations = readFileSync(join(packageRoot, "report-types.d.ts"), "utf8");
  assert.match(declarations, /queues: QueueCheckFinding\[\];/);
  assert.match(declarations, /rules: RuleFinding\[\];/);
  assert.match(declarations, /duplicates: DuplicateApiCall\[\];/);
  assert.match(declarations, /unsupported: UnsupportedApiCall\[\];/);
  assert.match(declarations, /fetches: ReactFetchCall\[\];/);
  assert.match(declarations, /children: ReactComponentRef\[\];/);
  assert.match(declarations, /inheritedFromChildren\?: ReactAggregatedFacts;/);
  assert.doesNotMatch(declarations, /queues: unknown\[\]/);
  assert.doesNotMatch(declarations, /duplicates: unknown\[\]/);
  assert.doesNotMatch(declarations, /fetches: unknown\[\]/);
});
