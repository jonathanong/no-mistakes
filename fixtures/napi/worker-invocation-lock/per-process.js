const assert = require("node:assert/strict");
const { execFileSync, fork } = require("node:child_process");
const { once } = require("node:events");
const { readFileSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { configureCompiledApiEnvironment } = require("../../../tests/js/compiled-api-environment.js");

// Exercise the non-Linux setup on Linux too, before loading the native addon.
configureCompiledApiEnvironment({
  platform: process.platform === "linux" ? "darwin" : process.platform,
});

async function main() {
  const api = require("../../../packages/no-mistakes/index.js");
  const root = join(__dirname, "../real-addon-dependencies");
  const expected = JSON.parse(
    readFileSync(join(__dirname, "../real-addon-dependencies/expected.json"), "utf8"),
  );
  const options = { root, files: ["entry.ts"], relationships: ["import"], failOnLock: true };
  // The first invocation creates the native process lock directory.
  assert.deepEqual(await api.dependencies(options), expected);
  const holder = fork(
    join(__dirname, "../planning-artifact-lock/hold.js"),
    [join(tmpdir(), "no-mistakes-test-locks", `${process.pid}.lock`)],
    { env: process.env, stdio: ["ignore", "ignore", "inherit", "ipc"] },
  );
  const exited = once(holder, "exit");
  try {
    const [message] = await Promise.race([
      once(holder, "message"),
      exited.then(([code, signal]) => {
        throw new Error(`lock holder exited before acquiring: ${code}/${signal}`);
      }),
    ]);
    assert.equal(message, "locked");
    await assert.rejects(
      api.dependencies(options),
      /another no-mistakes invocation is already running/,
    );
    const stdout = execFileSync(
      process.env["CARGO_BIN_EXE_no-mistakes"],
      [
        "dependencies",
        "entry.ts",
        "--root",
        root,
        "--relationship",
        "import",
        "--format",
        "json",
        "--fail-on-lock",
      ],
      { env: process.env, encoding: "utf8" },
    );
    assert.deepEqual(JSON.parse(stdout), expected);
    holder.send("release");
    assert.deepEqual(await exited, [0, null]);
    assert.deepEqual(await api.dependencies(options), expected);
    process.send({ success: true });
    process.disconnect();
  } finally {
    holder.kill();
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
  if (process.connected) process.disconnect();
});
