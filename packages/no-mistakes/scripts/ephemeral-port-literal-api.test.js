const assert = require("node:assert/strict");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

test(
  "compiled async check retains wrapped listen and signed Python zero findings",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(
      __dirname,
      "../../../test-cases/rules/no-raw-ephemeral-port/fixture/literal-recall",
    );
    const options = { root, config: join(root, ".no-mistakes.yml") };
    const pending = cjs.check(options);
    assert.ok(pending instanceof Promise);
    const report = await pending;
    const expected = [
      ...Array.from({ length: 6 }, (_, i) => ["server.py", i + 2]),
      ...Array.from({ length: 10 }, (_, i) => ["server.ts", i + 2]),
    ];
    assert.deepEqual(
      report.rules.map(({ file, line }) => [file, line]),
      expected,
    );
    assert.deepEqual(await esm.check(options), report);
  },
);
