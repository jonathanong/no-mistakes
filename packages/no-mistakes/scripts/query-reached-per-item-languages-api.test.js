const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled language call-policy findings match across CJS, ESM, and analyzeProject",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/query-reached-per-item/language-call-policy",
    );
    const report = await cjs.check({ root, config: "policy.yml" });
    const proven = report.rules.filter(
      (finding) => finding.rule === "query-reached-per-item" && finding.file === "python/worker.py",
    );
    assert.ok(
      proven.some(
        (finding) =>
          finding.message.includes("read_row") &&
          finding.message.includes("load → fetch → read_row") &&
          finding.import === "fetch",
      ),
      "python proven finding missing",
    );
    const advisories = report.advisories.filter(
      (finding) => finding.rule === "query-reached-per-item" && finding.file === "python/worker.py",
    );
    assert.ok(
      advisories.some(
        (finding) =>
          finding.message.includes("item.fetch") &&
          finding.causes?.[0]?.kind === "unresolved-dispatch",
      ),
      "python unproven advisory missing",
    );
    assert.deepEqual(await esm.check({ root, config: "policy.yml" }), report);
    const batch = await cjs.analyzeProject({
      root,
      reports: [{ type: "check", config: "policy.yml" }],
    });
    assert.deepEqual(batch.reports[0].result, report);
  },
);
