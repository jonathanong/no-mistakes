const { test, indexPath, planningPath, addonPath } = require("./context");
const { createMockAddon } = require("./mock-addon");
const { assertProxyQueries } = require("./proxy-queries");
const { assertProxyDomains } = require("./proxy-domains");

test("programmatic API proxies object options through async native addon calls", async () => {
  const collectionGap = {
    partial: true,
    incompleteFiles: [{ file: "broken.ts", reason: "parse-error", message: "Invalid syntax" }],
  };
  const previous = globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__;
  delete require.cache[require.resolve(indexPath)];
  delete require.cache[require.resolve(planningPath)];
  delete require.cache[addonPath];

  globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__ = createMockAddon(collectionGap);

  try {
    const api = require(indexPath);
    await assertProxyQueries(api, collectionGap);
    await assertProxyDomains(api);
  } finally {
    delete require.cache[require.resolve(indexPath)];
    delete require.cache[require.resolve(planningPath)];
    delete require.cache[addonPath];
    if (previous) {
      globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__ = previous;
    } else {
      delete globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__;
    }
  }
});
