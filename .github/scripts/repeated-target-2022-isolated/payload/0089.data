"use strict";

// A held real invocation lock makes the running libuv analysis deterministic:
// snapshot polling must complete before the analysis can be allowed to finish.
const assert = require("node:assert/strict");
const { fork } = require("node:child_process");
const { once } = require("node:events");
const { mkdirSync } = require("node:fs");
const { join } = require("node:path");
const { setTimeout: delay } = require("node:timers/promises");
process.env.XDG_RUNTIME_DIR = process.argv[2];
process.env.XDG_CACHE_HOME = process.argv[2];
const api = require("../../../packages/no-mistakes/index.js");

async function main() {
  const directory = join(process.argv[2], "no-mistakes");
  mkdirSync(directory, { recursive: true }); // Transient lock directory, not source fixtures.
  const child = fork(
    join(__dirname, "..", "planning-artifact-lock", "hold.js"),
    [join(directory, "invocation.lock")],
    {
      env: process.env,
      stdio: ["ignore", "ignore", "inherit", "ipc"],
    },
  );
  await once(child, "message");
  const count = Number(process.env.UV_THREADPOOL_SIZE);
  const options = Array.from({ length: count }, (_, id) => ({
    root: process.argv[3],
    diagnosticsId: `held-analysis-${id}`,
    lockTimeout: 3,
  }));
  const analyses = options.map((options) => api.check(options));
  let released = false;
  try {
    let snapshots;
    const end = Date.now() + 2000;
    while (Date.now() < end) {
      snapshots = await Promise.all(
        options.map(({ diagnosticsId: id }) => api.diagnosticsSnapshot({ id })),
      );
      if (
        snapshots.every((snapshot) =>
          snapshot?.active.some((span) => span.phase.includes("invocation.prepare")),
        )
      )
        break;
      await delay(5);
    }
    assert.ok(
      snapshots.every((snapshot) =>
        snapshot?.active.some((span) => span.phase.includes("invocation.prepare")),
      ),
    );
    assert.equal(await api.diagnosticsSnapshot({ id: "unrelated" }), null);
    const exited = once(child, "exit");
    child.send("release");
    await exited;
    released = true;
    await Promise.all(analyses);
    for (const { diagnosticsId: id } of options)
      assert.equal(await api.diagnosticsSnapshot({ id }), null);
    await assert.rejects(api.check({ root: 123, diagnosticsId: options[0].diagnosticsId }));
    assert.equal(await api.diagnosticsSnapshot({ id: options[0].diagnosticsId }), null);
  } finally {
    if (!released) child.send("release");
    await Promise.allSettled(analyses);
    child.kill();
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
