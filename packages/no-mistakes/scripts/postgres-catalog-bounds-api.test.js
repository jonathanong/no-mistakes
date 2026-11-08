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
      ["../issue-3114-exact/fixture", "r5.yml", 6],
      ["../issue-3114-exact/fixture", "r6.yml", 1],
      ["../materialized-bodies/fixture", "embedded.yml", 2],
      ["../materialized-bodies/fixture", "imported-report.yml", 1],
      ["../materialized-bodies/fixture", "imported-advise.yml", 0],
      ["column-policy", "../../configs/column-bounds-composite-prefix.yml", 1],
      ["column-policy", "../../configs/recursive-depth.yml", 10],
      ["join-roots", "valid.yml", 22],
      ["fixedpoint", "valid.yml", 27],
      ["issue-3037", "valid.yml", 3],
      ["issue-3037", "cte-valid.yml", 0],
      ["root-proofs", "valid.yml", 18],
      ["root-proofs", "invalid.yml", 19],
      ["../statement-groups/fixture", "valid.yml", 7],
      ["../statement-groups/fixture", "controls.yml", 6],
      ["../statement-groups/fixture", "embedded.yml", 4],
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
      if (config === "imported-advise.yml") {
        assert.equal(standalone.advisories.length, 1);
        assert.equal(standalone.advisories[0].file, "imported.ts");
        assert.equal(standalone.advisories[0].line, 5);
        assert.match(standalone.advisories[0].message, /MATERIALIZED CTE shared_pool/);
      }
      const relationTargets = {
        "key-resolution.yml": ["table:public.ambiguous_left", "table:public.ambiguous_right"],
        "key-operators.yml": ["table:public.customer_accounts", "table:public.owner_jobs"],
        "qualified-bare-tables.yml": ["table:a", "table:b"],
      };
      for (const target of relationTargets[config] || []) {
        assert.ok(
          standalone.rules.some((finding) => finding.target?.split(", ").includes(target)),
          `${config}: ${target}`,
        );
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
