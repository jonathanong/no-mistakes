const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const repository = join(__dirname, "../../..");

test(
  "compiled executor options preserve defaults and reject conflicting lists",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const source = readFileSync(
      join(repository, "fixtures/config/postgres-executor-names/source.ts"),
      "utf8",
    );
    const base = { sources: [{ fileName: "source.ts", source }], importSpecifier: "@example/db" };
    for (const api of [cjs, esm]) {
      for (const [options, count] of [
        [{ executorNames: ["run"] }, 1],
        [{ extraExecutorNames: ["run"] }, 4],
        [{ extraExecutorNames: [] }, 3],
      ]) {
        const pending = api.extractExecutedPostgresSql({ ...base, ...options });
        assert.equal(typeof pending.then, "function");
        assert.equal((await pending).length, count);
      }
      await assert.rejects(
        api.extractExecutedPostgresSql({ ...base, executorNames: [], extraExecutorNames: [] }),
        /executorNames and extraExecutorNames cannot both be set/,
      );
    }
  },
);

test(
  "compiled check exposes non-failing omitted executor warnings in CJS and ESM",
  { skip: !compiled },
  async () => {
    const cjs = require("../index.js");
    const esm = await import("../index.mjs");
    const root = join(repository, "fixtures/check/postgres-executor-names");
    for (const config of ["warning-only", "project-override"]) {
      const options = { root, config: join(root, `${config}.yml`) };
      const report = await cjs.check(options);
      assert.deepEqual(await esm.check(options), report);
      assert.deepEqual(report.rules, []);
      assert.equal(report.warnings.length, 1);
      assert.match(report.warnings[0], /executorNames.*`read`.*annotated.ts:2.*extraExecutorNames/);
    }
  },
);
