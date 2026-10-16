const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled effects alternatives share the CLI physical occurrence and category projections",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/codebase-analysis/effects-alternatives/fixture",
    );
    for (const projection of ["all", "read", "write"]) {
      const options = { root, kind: "database", entry: "src/entry.mts" };
      if (projection !== "all") options.categories = [projection];
      const expected = JSON.parse(readFileSync(join(root, `expected-${projection}.json`), "utf8"));
      const pending = cjs.effects(options);
      assert.equal(typeof pending.then, "function");
      assert.deepEqual(await pending, expected);
      assert.deepEqual(await esm.effects(options), expected);
      const { root: _root, ...report } = options;
      const batch = await cjs.analyzeProject({ root, reports: [{ type: "effects", ...report }] });
      assert.deepEqual(batch.reports[0].result, expected);
    }
  },
);

test(
  "compiled plain effects preserve alternatives and the no-write report in standalone and prepared calls",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const [root, kind, entry, expectedName] of [
      [
        join(
          __dirname,
          "../../../test-cases/rules/query-reached-per-item/plain-alternatives-fixture",
        ),
        "plain",
        "src/entry.mts",
        "expected-plain.json",
      ],
      [
        join(__dirname, "../../../test-cases/codebase-analysis/effects-plain-no-write/fixture"),
        "database",
        "src/defaults.mts",
        "expected.json",
      ],
    ]) {
      const expected = JSON.parse(readFileSync(join(root, expectedName), "utf8"));
      for (const api of [cjs, esm]) {
        const pending = api.effects({ root, kind, entry });
        assert.equal(typeof pending.then, "function");
        assert.deepEqual(await pending, expected);
        const batch = await api.analyzeProject({
          root,
          reports: [{ type: "effects", kind, entry }],
        });
        assert.deepEqual(batch.reports[0].result, expected);
      }
    }
  },
);
