const assert = require("node:assert/strict");
const { join } = require("node:path");

const packageRoot = join(__dirname, "..");
const addonPath = join(packageRoot, "bin", "no-mistakes.node");
const indexPath = join(packageRoot, "index.js");

test("programmatic API proxies object options through async native addon calls", async () => {
  const previous = require.extensions[".node"];
  delete require.cache[require.resolve(indexPath)];

  require.extensions[".node"] = (module, filename) => {
    assert.equal(filename, addonPath);
    module.exports = {
      dependenciesJson: async (json) =>
        JSON.stringify({ command: "dependencies", options: JSON.parse(json) }),
      dependentsJson: async (json) =>
        JSON.stringify({ command: "dependents", options: JSON.parse(json) }),
      relatedJson: async (json) =>
        JSON.stringify({ command: "related", options: JSON.parse(json) }),
      symbolsJson: async (json) =>
        JSON.stringify({ command: "symbols", options: JSON.parse(json) }),
      queuesJson: async (json) => JSON.stringify({ command: "queues", options: JSON.parse(json) }),
      queueEdgesJson: async (json) =>
        JSON.stringify({ command: "queueEdges", options: JSON.parse(json) }),
      queueRelatedJson: async (json) =>
        JSON.stringify({ command: "queueRelated", options: JSON.parse(json) }),
      queueCheckJson: async (json) =>
        JSON.stringify({ command: "queueCheck", options: JSON.parse(json) }),
      serverRoutesJson: async (json) =>
        JSON.stringify({ command: "serverRoutes", options: JSON.parse(json) }),
      serverRouteListJson: async (json) =>
        JSON.stringify({ command: "serverRouteList", options: JSON.parse(json) }),
      serverRouteEdgesJson: async (json) =>
        JSON.stringify({ command: "serverRouteEdges", options: JSON.parse(json) }),
      serverRouteRelatedJson: async (json) =>
        JSON.stringify({ command: "serverRouteRelated", options: JSON.parse(json) }),
      reactAnalyzeJson: async (json) =>
        JSON.stringify({ command: "reactAnalyze", options: JSON.parse(json) }),
      reactCheckJson: async (json) =>
        JSON.stringify({ command: "reactCheck", options: JSON.parse(json) }),
      version: async () => "1.2.3",
    };
  };

  try {
    const api = require(indexPath);
    assert.deepEqual(await api.dependencies({ files: ["a.mts"] }), {
      command: "dependencies",
      options: { files: ["a.mts"] },
    });
    assert.equal((await api.dependents({ files: ["b.mts"] })).command, "dependents");
    assert.equal((await api.related({ files: ["c.mts"] })).command, "related");
    assert.equal(
      (await api.symbols({ files: ["d.mts"], include: "both" })).options.include,
      "both",
    );
    assert.equal((await api.queues({ root: "." })).command, "queues");
    assert.equal((await api.queueEdges({ files: ["queue.ts"] })).command, "queueEdges");
    assert.equal((await api.queueRelated({ files: ["queue.ts"] })).command, "queueRelated");
    assert.equal((await api.queueCheck({ root: "." })).command, "queueCheck");
    assert.equal((await api.serverRoutes({ root: "." })).command, "serverRoutes");
    assert.equal((await api.serverRouteList({ files: ["/api"] })).command, "serverRouteList");
    assert.equal(
      (await api.serverRouteEdges({ roots: ["routes.ts"] })).command,
      "serverRouteEdges",
    );
    assert.equal(
      (await api.serverRouteRelated({ roots: ["routes.ts"] })).command,
      "serverRouteRelated",
    );
    assert.equal((await api.reactAnalyze({ targets: ["*.tsx"] })).command, "reactAnalyze");
    assert.equal((await api.reactCheck({ assertNoFetch: true })).command, "reactCheck");
    assert.equal(await api.version(), "1.2.3");
  } finally {
    delete require.cache[require.resolve(indexPath)];
    if (previous) {
      require.extensions[".node"] = previous;
    } else {
      delete require.extensions[".node"];
    }
  }
});
