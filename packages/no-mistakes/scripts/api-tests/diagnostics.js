require("./diagnostics-errors");
const { assert, test } = require("./context");
const { fixtures, withApi, envelope } = require("./diagnostics-context");

test("collected native diagnostics preserve normal query and planning postprocessing", async () => {
  const inputs = [];
  const native = (result) => async (input) => {
    inputs.push(JSON.parse(input));
    return envelope(result);
  };
  await withApi(
    {
      checkJson: native({ ok: true }),
      testsPlanJson: native(fixtures.plan),
      analyzeProjectJson: native(fixtures.project),
    },
    async (api) => {
      const options = { root: "fixture-root" };
      assert.deepEqual(await api.collectDiagnostics("check", options), {
        result: { ok: true },
        diagnostics: fixtures.diagnostics,
      });
      assert.deepEqual(await api.collectDiagnostics("testsPlan", options), {
        result: { changedFiles: ["src/example.ts"], testFiles: ["test/example.test.ts"] },
        diagnostics: fixtures.diagnostics,
      });
      const project = await api.collectDiagnostics("analyzeProject", options);
      assert.deepEqual(project.result.reports[0].result, { changedFiles: ["src/example.ts"] });
      assert.equal(options.__collectDiagnostics, undefined);
      assert.ok(inputs.every((input) => input.__collectDiagnostics === true));
    },
  );
});

test("ordinary API calls do not opt into collection before or after a collected call", async () => {
  const requests = [];
  await withApi(
    {
      checkJson: async (input) => {
        const options = JSON.parse(input);
        requests.push(options);
        return options.__collectDiagnostics ? envelope(options) : JSON.stringify(options);
      },
    },
    async (api) => {
      assert.deepEqual(await api.check({ root: "ordinary" }), { root: "ordinary" });
      await api.collectDiagnostics("check", { root: "collected" });
      assert.deepEqual(await api.check(), {});
    },
  );
  assert.deepEqual(requests, [
    { root: "ordinary" },
    { root: "collected", __collectDiagnostics: true },
    {},
  ]);
});

test("concurrent collection and an ordinary call remain isolated", async () => {
  let releaseFirst;
  const firstPending = new Promise((resolve) => {
    releaseFirst = resolve;
  });
  let enteredFirst;
  const firstEntered = new Promise((resolve) => {
    enteredFirst = resolve;
  });
  await withApi(
    {
      checkJson: async (input) => {
        const options = JSON.parse(input);
        if (options.root === "first") {
          enteredFirst();
          await firstPending;
        }
        if (!options.__collectDiagnostics) return JSON.stringify(options);
        return envelope(options.root, {
          ...fixtures.diagnostics,
          elapsedMs: options.root === "first" ? 1 : 2,
        });
      },
    },
    async (api) => {
      const first = api.collectDiagnostics("check", { root: "first" });
      await firstEntered;
      try {
        const second = await api.collectDiagnostics("check", { root: "second" });
        assert.equal(second.result, "second");
        assert.equal(second.diagnostics.elapsedMs, 2);
        assert.deepEqual(await api.check({ root: "ordinary" }), { root: "ordinary" });
      } finally {
        releaseFirst();
      }
      const completed = await first;
      assert.equal(completed.result, "first");
      assert.equal(completed.diagnostics.elapsedMs, 1);
    },
  );
});

test("nested collection restores the outer diagnostics context", async () => {
  let api;
  let inner;
  const native = {
    checkJson: async () => {
      inner = await api.collectDiagnostics("dependencies", { root: "inner" });
      return envelope("outer", { ...fixtures.diagnostics, elapsedMs: 1 });
    },
    dependenciesJson: async () => envelope("inner", { ...fixtures.diagnostics, elapsedMs: 2 }),
  };
  await withApi(native, async (loaded) => {
    api = loaded;
    const outer = await api.collectDiagnostics("check", {});
    assert.equal(outer.result, "outer");
    assert.equal(outer.diagnostics.elapsedMs, 1);
    assert.equal(inner.result, "inner");
    assert.equal(inner.diagnostics.elapsedMs, 2);
  });
});

test("collection preserves batch array and scalar native inputs", async () => {
  const requests = [];
  await withApi(
    {
      parsePostgresSqlJson: async (input) => {
        const request = JSON.parse(input);
        requests.push(request);
        return request.__collectDiagnostics
          ? envelope(request.__diagnosticsInput)
          : JSON.stringify(request);
      },
    },
    async (api) => {
      const batch = ["select 1", "select 2"];
      assert.deepEqual(await api.parsePostgresSql(batch), batch);
      assert.deepEqual((await api.collectDiagnostics("parsePostgresSql", batch)).result, batch);
      assert.equal(
        (await api.collectDiagnostics("parsePostgresSql", "select 3")).result,
        "select 3",
      );
      assert.deepEqual(batch, ["select 1", "select 2"]);
    },
  );
  assert.deepEqual(requests, [
    ["select 1", "select 2"],
    { __collectDiagnostics: true, __diagnosticsInput: ["select 1", "select 2"] },
    { __collectDiagnostics: true, __diagnosticsInput: "select 3" },
  ]);
});
