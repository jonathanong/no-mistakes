const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled argument predicates share async CLI-equivalent CJS/ESM/prepared checks",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/forbidden-calls/argument-predicates/fixture",
    );
    const options = { root, config: join(root, ".no-mistakes.yml") };
    const pending = cjs.check(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    assert.deepEqual(await esm.check(options), report);
    assert.deepEqual(
      report.rules.map((finding) => [finding.file, finding.line, finding.target]),
      [
        ["repro.mjs", 14, "module export `node:child_process#execFile`"],
        ["repro.mjs", 15, "module export `node:child_process#execFile`"],
      ],
    );
    assert.ok(
      report.rules.every((finding) => finding.message.includes("no-live-analyzer-from-tests")),
    );
    const aggregate = await cjs.analyzeProject({ ...options, reports: [{ type: "check" }] });
    assert.deepEqual(aggregate.reports[0].result, report);
  },
);

test(
  "compiled Exact file checks preserve canonical calls for a combined query",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/forbidden-calls/argument-predicates/fixture",
    );
    const options = { root, config: join(root, "exported-calls-exact.yml") };
    const expected = await cjs.check(options);
    assert.deepEqual(
      expected.rules.map(({ file, line }) => [file, line]),
      [["exported-calls.mjs", 2]],
    );
    assert.deepEqual(await esm.check(options), expected);
    const query = { type: "callSites", file: "exported-calls.mjs", exportName: "invoke" };
    const standaloneCalls = await cjs.callSites({
      root,
      file: query.file,
      exportName: query.exportName,
    });
    assert.ok(standaloneCalls.callSites.length > 0);
    for (const api of [cjs, esm]) {
      const pending = api.analyzeProject({ ...options, reports: [{ type: "check" }, query] });
      assert.ok(pending instanceof Promise);
      const batch = await pending;
      assert.deepEqual(batch.reports[0].result, expected);
      assert.deepEqual(batch.reports[1].result, standaloneCalls);
    }
  },
);
