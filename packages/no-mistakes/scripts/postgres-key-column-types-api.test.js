const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled CJS, ESM and prepared checks share catalog key-type findings",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const fixture = join(__dirname, "../../../test-cases/rules/postgres-key-column-types/fixture");
    for (const scenario of ["pass", "fail", "fail-stale-allow", "pass-suppressed"]) {
      const root = join(fixture, scenario);
      const pending = cjs.check({ root });
      assert.ok(pending instanceof Promise);
      const report = await pending;
      assert.deepEqual(report.warnings, []);
      assert.deepEqual(await esm.check({ root }), report);
      assert.deepEqual(await cjs.check({ root }), report, "catalog findings must be deterministic");
      const batch = await cjs.analyzeProject({
        root,
        reports: [{ type: "check", id: "key-types" }],
      });
      assert.equal(batch.reports[0].id, "key-types");
      assert.deepEqual(batch.reports[0].result, report);
      if (scenario.startsWith("pass")) {
        assert.deepEqual(report.rules, []);
        continue;
      }
      assert.ok(report.rules.length > 0);
      for (const finding of report.rules) {
        assert.equal(finding.rule, "postgres-key-column-types");
        assert.equal(finding.file, "schema.json");
        assert.equal(finding.line, 1);
        assert.match(finding.target, /^constraint:/);
        assert.ok(finding.message.includes(finding.target));
      }
      if (scenario === "fail-stale-allow") {
        assert.ok(
          report.rules.some((finding) =>
            finding.message.includes("stale postgres-key-column-types allow entry"),
          ),
        );
      } else {
        assert.ok(report.rules.some((finding) => finding.message.includes("primary key uses")));
        assert.ok(report.rules.some((finding) => finding.message.includes("foreign key uses")));
        const byTarget = new Map(report.rules.map((finding) => [finding.target, finding]));
        assert.equal(
          byTarget.size,
          4,
          "each key gets one finding and partition leaves are skipped",
        );
        assert.match(
          byTarget.get("constraint:composite_orders.composite_orders_pkey").message,
          /primary key uses text column tag, character varying column locale;/,
        );
        assert.match(
          byTarget.get("constraint:legacy_order.legacy_order_fkey").message,
          /foreign key uses text column id \(references orders\)/,
        );
        for (const parent of ["partitioned_parent", "partitioned_subparent"]) {
          assert.ok(byTarget.has(`constraint:${parent}.${parent}_pkey`));
        }
      }
    }
  },
);

test(
  "compiled key-type diagnostics preserve numeric precision and exact configured matching",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-key-column-types/fixture/fail-display-types",
    );
    const target = "constraint:numeric_precision.numeric_precision_pkey";
    for (const allowed of [false, true]) {
      const options = allowed
        ? { root, config: join(root, ".no-mistakes-numeric-allowed.yml") }
        : { root };
      const report = await cjs.check(options);
      assert.deepEqual(report.warnings, []);
      assert.deepEqual(await esm.check(options), report);
      const batch = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
      assert.deepEqual(batch.reports[0].result, report);
      const finding = report.rules.find((entry) => entry.target === target);
      if (allowed) {
        assert.equal(finding, undefined);
        assert.equal(report.rules.length, 2);
      } else {
        assert.equal(report.rules.length, 3);
        assert.equal(finding.file, "schema.json");
        assert.equal(finding.line, 1);
        assert.match(finding.message, /primary key uses numeric\(10,2\) column id;/);
      }
    }
  },
);

test(
  "compiled key-type checks reject catalog keys with empty column lists",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const fixture = join(__dirname, "../../../test-cases/rules/postgres-key-column-types/fixture");
    for (const [scenario, detail] of [
      ["fail-empty-primary", "primary key orders.orders_pkey has no columns"],
      ["fail-empty-foreign", "foreign key orders.invalid_fkey has no columns"],
    ]) {
      const root = join(fixture, scenario);
      const validate = (error) => {
        assert.ok(error.message.includes("schemaCatalogPath schema.json"));
        assert.ok(error.message.includes(detail));
        assert.ok(error.message.includes("regenerate the schema catalog"));
        return true;
      };
      await assert.rejects(cjs.check({ root }), validate);
      await assert.rejects(esm.check({ root }), validate);
      await assert.rejects(cjs.analyzeProject({ root, reports: [{ type: "check" }] }), validate);
    }
  },
);
