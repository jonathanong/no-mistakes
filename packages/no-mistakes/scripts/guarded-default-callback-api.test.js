const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled async default callbacks preserve capture generations",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/value-flow-guarded-parameters/fixture",
    );
    const options = {
      root,
      config: join(root, "effects.yml"),
      reports: ["before", "after"].map((suffix) => ({
        type: "effects",
        kind: "receiver",
        entry: `src/array-parameter-default-callback-${suffix}.mts`,
      })),
    };
    const pending = cjs.analyzeProject(options);
    assert.ok(pending instanceof Promise);
    const batch = await pending;
    for (const [index, retained] of ["replica.get", "untouched.get"].entries()) {
      // The default callable must receive its input before the capture is frozen.
      assert.deepEqual(
        batch.reports[index].result.callSites.map(({ callee }) => callee),
        [retained, "primary.get", "replica.info", "unrelated.get"],
      );
    }
    assert.deepEqual(await esm.analyzeProject(options), batch);
  },
);

test(
  "compiled async bound array spreads retain candidates and exact empty controls",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/value-flow-optional-prefixes/fixture",
    );
    const entries = [
      "array-bind-varying-width",
      "array-bind-optional-push",
      "array-bind-known-empty",
    ];
    const options = {
      root,
      config: join(root, "effects.yml"),
      reports: entries.map((entry) => ({
        type: "effects",
        kind: "receiver",
        entry: `src/${entry}.mts`,
      })),
    };
    const pending = cjs.analyzeProject(options);
    assert.ok(pending instanceof Promise);
    const batch = await pending;
    for (const [index, { result }] of batch.reports.entries()) {
      assert.deepEqual(
        result.callSites.map(({ callee }) => callee),
        index === 2
          ? ["replica.get", "primary.get", "replica.info", "unrelated.get"]
          : ["replica.get", "untouched.get", "primary.get", "replica.info", "unrelated.get"],
      );
    }
    assert.deepEqual(await esm.analyzeProject(options), batch);
  },
);
