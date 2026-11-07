const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled checks group PostgreSQL statement and expression sites",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const cases = [
      ["postgres-bounded-statements/statement-groups/fixture", "valid.yml", 7],
      ["postgres-bounded-statements/statement-groups/fixture", "controls.yml", 6],
      ["postgres-bounded-statements/statement-groups/fixture", "embedded.yml", 4],
      ["postgres-bounded-statements/statement-groups/fixture", "variant-prefix.yml", 1],
      ["postgres-bounded-statements/statement-groups/fixture", "repeated-helper.yml", 2],
      [
        "postgres-no-transformed-uuid-predicates/fixture",
        "../configs/multi-argument-sites.yml",
        17,
      ],
      [
        "postgres-no-transformed-uuid-predicates/fixture",
        "../configs/multi-argument-sites-low.yml",
        17,
      ],
      [
        "postgres-no-transformed-uuid-predicates/fixture",
        "../configs/multi-argument-variants.yml",
        1,
      ],
      [
        "postgres-no-transformed-uuid-predicates/fixture",
        "../configs/multi-argument-variants-low.yml",
        1,
      ],
      [
        "postgres-no-transformed-uuid-predicates/fixture",
        "../configs/multi-argument-shared-column.yml",
        2,
      ],
      [
        "postgres-no-transformed-uuid-predicates/fixture",
        "../configs/multi-argument-shared-column-low.yml",
        2,
      ],
      [
        "postgres-no-transformed-uuid-predicates/fixture",
        "../configs/multi-argument-shared-helper.yml",
        1,
      ],
      [
        "postgres-no-transformed-uuid-predicates/fixture",
        "../configs/multi-argument-shared-helper-low.yml",
        1,
      ],
      [
        "postgres-no-transformed-uuid-predicates/fixture",
        "../configs/multi-argument-shared-casts.yml",
        13,
      ],
      [
        "postgres-no-transformed-uuid-predicates/fixture",
        "../configs/multi-argument-shared-casts-low.yml",
        13,
      ],
    ];
    for (const [fixture, config, expected] of cases) {
      const root = join(__dirname, "../../../test-cases/rules", fixture);
      const options = { root, config };
      const check = await cjs.check(options);
      assert.equal(check.rules.length, expected, config);
      assert.deepEqual(await esm.check(options), check, config);
      const project = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
      assert.deepEqual(project.reports[0].result, check, config);
      assert.deepEqual(check.warnings, [], config);
    }
  },
);
