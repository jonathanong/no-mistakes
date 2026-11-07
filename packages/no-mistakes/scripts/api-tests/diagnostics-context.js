const {
  join,
  repositoryRoot,
  indexPath,
  planningPath,
  addonPath,
  readFileSync,
} = require("./context");
const fixtures = JSON.parse(
  readFileSync(join(repositoryRoot, "fixtures/napi/collect-diagnostics/reports.json"), "utf8"),
);

async function withApi(native, operation) {
  const previous = globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__;
  const previousPath = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
  for (const file of [indexPath, planningPath, addonPath])
    delete require.cache[require.resolve(file)];
  globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__ = native;
  process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH = addonPath;
  try {
    return await operation(require(indexPath));
  } finally {
    for (const file of [indexPath, planningPath, addonPath])
      delete require.cache[require.resolve(file)];
    if (previous === undefined) delete globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__;
    else globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__ = previous;
    if (previousPath === undefined) delete process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
    else process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH = previousPath;
  }
}
const envelope = (result, diagnostics = fixtures.diagnostics) =>
  JSON.stringify({ result, diagnostics });

module.exports = { fixtures, withApi, envelope };
