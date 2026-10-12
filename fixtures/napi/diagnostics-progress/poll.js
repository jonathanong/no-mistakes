"use strict";

// A held real invocation lock makes the running libuv analysis deterministic:
// snapshot polling must complete before the analysis can be allowed to finish.
const assert = require("node:assert/strict");
const { mkdirSync } = require("node:fs");
const { join } = require("node:path");
const { setTimeout: delay } = require("node:timers/promises");
process.env.XDG_RUNTIME_DIR = process.argv[2];
process.env.XDG_CACHE_HOME = process.argv[2];
const native = require(process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH);
const api = require("../../../packages/no-mistakes/index.js");

async function main() {
  const directory = join(process.argv[2], "no-mistakes");
  mkdirSync(directory, { recursive: true }); // Transient lock directory, not source fixtures.
  const token = await native.acquirePlanningArtifactLock(join(directory, "invocation.lock"));
  const options = { root: process.argv[3], diagnosticsId: "held-analysis", lockTimeout: 3 };
  const analysis = api.check(options);
  let released = false;
  try {
    let snapshot;
    const end = Date.now() + 2000;
    while (Date.now() < end) {
      snapshot = await api.diagnosticsSnapshot({ id: options.diagnosticsId });
      if (snapshot?.active.some((span) => span.phase.includes("invocation.prepare"))) break;
      await delay(5);
    }
    assert.ok(snapshot?.active.some((span) => span.phase.includes("invocation.prepare")));
    assert.equal(await api.diagnosticsSnapshot({ id: "unrelated" }), null);
    await assert.rejects(api.check(options), /already active/);
    await native.releasePlanningArtifactLock(token);
    released = true;
    await analysis;
    assert.equal(await api.diagnosticsSnapshot({ id: options.diagnosticsId }), null);
    await assert.rejects(api.check({ root: 123, diagnosticsId: options.diagnosticsId }));
    assert.equal(await api.diagnosticsSnapshot({ id: options.diagnosticsId }), null);
  } finally {
    if (!released) await native.releasePlanningArtifactLock(token);
    await analysis.catch(() => {});
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
