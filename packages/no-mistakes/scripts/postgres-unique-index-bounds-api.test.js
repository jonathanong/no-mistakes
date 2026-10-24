const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled checks preserve unique-index bound proofs across CJS, ESM and project reports",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/postgres-bounded-statements/fixture/unique-index-shapes",
    );
    const options = { root, config: ".no-mistakes.yml" };
    const standalone = await cjs.check(options);
    assert.deepEqual(await esm.check(options), standalone);
    const project = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
    assert.deepEqual(project.reports[0].result, standalone);
    assert.deepEqual(standalone.warnings, []);
    assert.equal(standalone.rules.length, 24);
    assert.ok(
      standalone.rules.some(
        (finding) => finding.file === "partial-unique.sql" && finding.line === 3,
      ),
    );
    assert.ok(
      !standalone.rules.some(
        (finding) => finding.file === "partial-unique.sql" && finding.line === 2,
      ),
    );
  },
);
