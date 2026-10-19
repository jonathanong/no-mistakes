const {
  assert,
  test,
  join,
  pathToFileURL,
  packageRoot,
  indexPath,
  planningPath,
  addonPath,
  RAW_NATIVE_EXPORTS,
  PURE_JAVASCRIPT_EXPORTS,
  nativeExportNames,
  declarationExportNames,
  nativeExportNameForApi,
} = require("./context");

test("native exports, JavaScript exports, and declarations stay in parity", async () => {
  const previous = globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__;
  const nativeExports = nativeExportNames();
  const observedExports = new Set();
  delete require.cache[require.resolve(indexPath)];
  delete require.cache[require.resolve(planningPath)];
  delete require.cache[addonPath];

  globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__ = Object.fromEntries(
    nativeExports.map((name) => [
      name,
      async (json) => {
        observedExports.add(name);
        if (Object.values(RAW_NATIVE_EXPORTS).includes(name)) return name;
        return JSON.stringify({ name, options: JSON.parse(json) });
      },
    ]),
  );

  try {
    const api = require(indexPath);
    const declaredExports = declarationExportNames();
    assert.deepEqual(Object.keys(api).sort(), declaredExports);

    // These exports are intentionally pure JS; every other declared function
    // must cross the N-API boundary and keep returning a promise.
    for (const name of declaredExports) {
      if (PURE_JAVASCRIPT_EXPORTS.has(name)) continue;
      const expectedNativeExport = nativeExportNameForApi(name);
      const result = api[name]({});
      assert.equal(typeof result.then, "function", `${name} must remain async`);
      const value = await result;
      if (Object.hasOwn(RAW_NATIVE_EXPORTS, name)) {
        assert.equal(value, expectedNativeExport);
      } else {
        assert.equal(value.name, expectedNativeExport);
      }
    }

    assert.deepEqual([...observedExports].sort(), nativeExports);
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

test("native ESM imports expose every declared root API", async () => {
  const previous = globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__;
  delete require.cache[require.resolve(indexPath)];
  delete require.cache[require.resolve(planningPath)];
  delete require.cache[require.resolve(addonPath)];

  globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__ = { version: async () => "1.2.3" };

  try {
    const api = require(indexPath);
    // Import the package's actual ESM wrapper, rather than Node's CJS namespace.
    const esm = await import(
      `${pathToFileURL(join(packageRoot, "index.mjs")).href}?exports=${Date.now()}`
    );
    const declaredExports = declarationExportNames();
    assert.deepEqual(
      Object.keys(esm)
        .filter((name) => name !== "default")
        .sort(),
      declaredExports,
    );
    assert.equal(esm.default, api);

    for (const name of declaredExports) {
      assert.equal(typeof esm[name], "function", `${name} must support native ESM named imports`);
      assert.equal(esm[name], api[name], `${name} must match the CommonJS export`);
    }

    assert.equal(typeof esm.ciTopology, "function");
    assert.equal(typeof esm.ciTopologyImpact, "function");
    assert.equal(typeof esm.testsPlan, "function");
    assert.equal(typeof esm.createWorkflowTopologyIndex, "function");
    assert.equal(typeof esm.version, "function");
  } finally {
    delete require.cache[require.resolve(indexPath)];
    delete require.cache[require.resolve(planningPath)];
    delete require.cache[require.resolve(addonPath)];
    if (previous) {
      globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__ = previous;
    } else {
      delete globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__;
    }
  }
});
