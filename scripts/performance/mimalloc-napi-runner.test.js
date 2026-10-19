"use strict";

const assert = require("node:assert/strict");
const test = require("node:test");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const reports = require("../../fixtures/performance/mimalloc/napi/reports.json");
const { checkExitCode, run } = require("./mimalloc-napi-runner");

test("exit status follows CLI failure domains and preserves advisory-only success", () => {
  assert.equal(checkExitCode(reports.clear), 0);
  assert.equal(checkExitCode(reports.advisoryOnly), 0);
  assert.equal(checkExitCode(reports.warning), 1);
  for (const domain of ["rule", "react", "queues", "integration", "codebase"]) {
    assert.equal(checkExitCode(reports[domain]), 1);
  }
  assert.throws(() => checkExitCode(reports.incomplete), /missing check report array/);
});

test("awaits the facade, disables both inner deadlines, and writes the complete report", async () => {
  const original = process.stdout.write;
  let output = "";
  process.stdout.write = (chunk) => {
    output += chunk;
    return true;
  };
  try {
    let options;
    const check = async (request) => {
      options = request;
      return reports.warning;
    };
    assert.equal(await run(check, ".", "1"), 1);
    assert.deepEqual(options, { root: path.resolve("."), timeout: 0, lockTimeout: 0, jobs: 1 });
    assert.deepEqual(JSON.parse(output), reports.warning);
    await run(check, ".", "max");
    assert.equal("jobs" in options, false);
    await assert.rejects(run(check, ".", "4"), /jobs must be/);
    await assert.rejects(
      run(
        async () => {
          throw new Error("native failure");
        },
        ".",
        "1",
      ),
      /native failure/,
    );
  } finally {
    process.stdout.write = original;
  }
});

test("runner reports setup errors with status 2", () => {
  const env = { ...process.env };
  delete env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
  const child = spawnSync(
    process.execPath,
    [path.join(__dirname, "mimalloc-napi-runner.js"), ".", "1"],
    { env, encoding: "utf8" },
  );
  assert.equal(child.status, 2);
  assert.match(child.stderr, /explicit measured addon required/);
  assert.equal(child.stdout, "");
});
