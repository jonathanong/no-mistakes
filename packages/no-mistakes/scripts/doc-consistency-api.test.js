const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

function optionsFor(name) {
  const root = join(__dirname, "../../../test-cases/rules/doc-consistency/fixture", name);
  return { root, config: join(root, ".no-mistakes.yml") };
}

async function rejection(promise) {
  try {
    await promise;
  } catch (error) {
    return error;
  }
  throw new Error("expected rejection");
}

test(
  "doc-consistency shares async CJS/ESM/prepared pattern checks",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const options = optionsFor("patterns");
    const pending = cjs.check(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(await esm.check(options), report);
    const aggregate = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
    assert.deepEqual(aggregate.reports[0].result, report);
    assert.deepEqual(
      (await esm.analyzeProject({ ...options, reports: [{ type: "check" }] })).reports[0].result,
      report,
    );
    const rules = report.rules.filter((finding) => finding.rule === "doc-consistency");
    assert.ok(
      rules.some(
        (finding) =>
          finding.file === "docs/living.md" &&
          finding.line === 3 &&
          finding.message === "Move this pin to versions.md.",
      ),
    );
    assert.ok(!rules.some((finding) => finding.file === "docs/file-disabled.md"));
    assert.ok(
      !rules.some((finding) => finding.file === "docs/line-disabled.md" && finding.line === 2),
    );
    assert.ok(
      rules.some((finding) => finding.file === "docs/line-disabled.md" && finding.line === 3),
    );
  },
);

test(
  "doc-consistency rejects an invalid pattern from CJS and ESM",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const options = optionsFor("patterns-invalid");
    const cjsError = await rejection(cjs.check(options));
    const esmError = await rejection(esm.check(options));
    const preparedError = await rejection(
      cjs.analyzeProject({ ...options, reports: [{ type: "check" }] }),
    );
    const esmPreparedError = await rejection(
      esm.analyzeProject({ ...options, reports: [{ type: "check" }] }),
    );
    for (const error of [cjsError, esmError, preparedError, esmPreparedError]) {
      assert.match(String(error), /bannedPatterns\[0\]/);
      assert.match(String(error), /invalid regex/);
    }
    assert.equal(String(cjsError), String(esmError));
    assert.equal(String(preparedError), String(esmPreparedError));
  },
);
