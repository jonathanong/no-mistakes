const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled catalog bounds agree across CJS, ESM and project reports",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    for (const [directory, config, expected] of [
      ["catalog-resolution", "key-resolution.yml", 5],
      ["catalog-resolution", "key-operators.yml", 2],
      ["catalog-resolution", "key-booleans.yml", 1],
      ["catalog-resolution", "view-bounds.yml", 1],
      ["catalog-resolution", "view-allow.yml", 0],
      ["catalog-resolution", "view-suppression.yml", 0],
      ["catalog-resolution-identity", "view-identity.yml", 1],
      ["schemaless-key-resolution", "qualified-bare-tables.yml", 3],
      ["column-policy", "../../configs/column-bounds.yml", 6],
      ["column-policy", "../../configs/column-bounds-composite-prefix.yml", 1],
      ["column-policy", "../../configs/recursive-depth.yml", 10],
      ["../variants/correlated-append", ".no-mistakes.yml", 1],
      ["../variants/correlated-append", "undeclared.yml", 2],
      ["../variants/correlated-append", "advise.yml", 0],
    ]) {
      const root = join(
        __dirname,
        "../../../test-cases/rules/postgres-bounded-statements/fixture",
        directory,
      );
      const options = { root, config, includeSqlVerdicts: true };
      const standalone = await cjs.check(options);
      assert.equal(standalone.rules.length, expected, config);
      const groupedTargets = {
        "key-resolution.yml": "table:public.ambiguous_left, table:public.ambiguous_right",
        "key-operators.yml": "table:public.customer_accounts, table:public.owner_jobs",
        "qualified-bare-tables.yml": "table:a, table:b",
      };
      if (groupedTargets[config]) {
        assert.ok(standalone.rules.some((finding) => finding.target === groupedTargets[config]));
      }
      assert.deepEqual(await esm.check(options), standalone, config);
      const project = await cjs.analyzeProject({
        root,
        config,
        reports: [{ type: "check", includeSqlVerdicts: true }],
      });
      assert.deepEqual(project.reports[0].result, standalone, config);
      if (config === "view-identity.yml") {
        assert.ok(standalone.sqlVerdicts.some((row) => row.status === "abstain"));
      }
    }
  },
);
