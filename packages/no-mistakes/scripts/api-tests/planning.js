const {
  assert,
  test,
  existsSync,
  readFileSync,
  writeFileSync,
  mkdtempSync,
  join,
  tmpdir,
  indexPath,
  planningPath,
  addonPath,
  planningFixturePath,
} = require("./context");

test("testsWhy and analyzeProject clean generated why-plan directories", async () => {
  const previous = globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__;
  const seen = [];
  delete require.cache[require.resolve(indexPath)];
  delete require.cache[require.resolve(planningPath)];
  delete require.cache[addonPath];
  globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__ = {
    testsWhyJson: async (json) => {
      const options = JSON.parse(json);
      seen.push(options.plan);
      if (options.test === "__reject__") throw new Error("why failed");
      return JSON.stringify({ command: "testsWhy", options });
    },
    analyzeProjectJson: async (json) => {
      const options = JSON.parse(json);
      for (const report of options.reports || []) seen.push(report.plan);
      if (options.reports?.some((report) => report.test === "__reject__")) {
        throw new Error("why failed");
      }
      return JSON.stringify({ command: "analyzeProject", options });
    },
  };
  try {
    const api = require(indexPath);
    const planDir = mkdtempSync(join(tmpdir(), "no-mistakes-plan-"));
    const planPath = join(planDir, "plan.json");
    writeFileSync(planPath, JSON.stringify({ selectedTests: [] }));
    await api.testsWhy({ test: "source.test.ts", planJson: { selectedTests: [] } });
    await api.testsWhy({ test: "source.test.ts", plan: planPath });
    await api.analyzeProject({
      reports: [
        { type: "testsWhy", test: "batched.test.ts", planJson: { selectedTests: [] } },
        { type: "testsWhy", test: "file.test.ts", plan: planPath },
      ],
    });
    await assert.rejects(
      api.testsWhy({ test: "__reject__", planJson: { selectedTests: [] } }),
      /why failed/,
    );
    await assert.rejects(
      api.analyzeProject({
        reports: [{ type: "testsWhy", test: "__reject__", planJson: { selectedTests: [] } }],
      }),
      /why failed/,
    );
    assert.equal(seen.length, 6);
    for (const plan of seen) {
      assert.match(plan, /no-mistakes-why-/);
      assert.equal(existsSync(plan), false);
    }
    assert.equal(existsSync(planPath), true);
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

test("preparation write failures clean their temporary directory and preserve the error", async () => {
  const previous = globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__;
  const fs = require("node:fs/promises");
  const originalMkdtemp = fs.mkdtemp;
  const originalWriteFile = fs.writeFile;
  const generatedDirs = [];
  const injected = new Error("plan write failed");
  delete require.cache[require.resolve(indexPath)];
  delete require.cache[require.resolve(planningPath)];
  delete require.cache[addonPath];
  globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__ = {
    testsWhyJson: async () => JSON.stringify({ command: "testsWhy" }),
  };
  fs.mkdtemp = async (...args) => {
    const generatedDir = await originalMkdtemp(...args);
    generatedDirs.push(generatedDir);
    return generatedDir;
  };
  fs.writeFile = async (file, ...args) => {
    if (String(file).includes("no-mistakes-why-")) throw injected;
    return originalWriteFile(file, ...args);
  };
  try {
    const api = require(indexPath);
    await assert.rejects(
      api.testsWhy({ planJson: JSON.parse(readFileSync(planningFixturePath, "utf8")) }),
      (error) => error === injected,
    );
    assert.equal(generatedDirs.length, 1);
    assert.equal(existsSync(generatedDirs[0]), false);
  } finally {
    fs.mkdtemp = originalMkdtemp;
    fs.writeFile = originalWriteFile;
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

test("analyzeProject waits for every preparation before cleaning generated plans", async () => {
  const previous = globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__;
  const fs = require("node:fs/promises");
  const originalMkdtemp = fs.mkdtemp;
  const originalWriteFile = fs.writeFile;
  const generatedDirs = [];
  const failed = new Error("first plan write failed");
  let writeCount = 0;
  let releaseSibling;
  let siblingStarted;
  const siblingStartedPromise = new Promise((resolve) => {
    siblingStarted = resolve;
  });
  const siblingReleased = new Promise((resolve) => {
    releaseSibling = resolve;
  });
  delete require.cache[require.resolve(indexPath)];
  delete require.cache[require.resolve(planningPath)];
  delete require.cache[addonPath];
  globalThis.__NO_MISTAKES_TEST_NAPI_ADDON__ = {
    analyzeProjectJson: async () => JSON.stringify({ command: "analyzeProject", reports: [] }),
  };
  fs.mkdtemp = async (...args) => {
    const generatedDir = await originalMkdtemp(...args);
    generatedDirs.push(generatedDir);
    return generatedDir;
  };
  fs.writeFile = async (file, ...args) => {
    if (!String(file).includes("no-mistakes-why-")) return originalWriteFile(file, ...args);
    writeCount += 1;
    if (writeCount === 1) throw failed;
    siblingStarted();
    await siblingReleased;
    return originalWriteFile(file, ...args);
  };
  try {
    const api = require(indexPath);
    const request = api.analyzeProject({
      reports: [
        { type: "testsWhy", planJson: JSON.parse(readFileSync(planningFixturePath, "utf8")) },
        { type: "testsWhy", planJson: JSON.parse(readFileSync(planningFixturePath, "utf8")) },
      ],
    });
    await siblingStartedPromise;
    let settled = false;
    void request.then(
      () => {
        settled = true;
      },
      () => {
        settled = true;
      },
    );
    await new Promise((resolve) => setImmediate(resolve));
    assert.equal(settled, false);
    releaseSibling();
    await assert.rejects(request, (error) => error === failed);
    assert.equal(generatedDirs.length, 2);
    for (const generatedDir of generatedDirs) assert.equal(existsSync(generatedDir), false);
  } finally {
    releaseSibling();
    fs.mkdtemp = originalMkdtemp;
    fs.writeFile = originalWriteFile;
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
