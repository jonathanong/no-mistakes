const assert = require("node:assert/strict");
const { fork } = require("node:child_process");
const { once } = require("node:events");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;
require("../../packages/no-mistakes/scripts/workflow-secret-inventory-api.test.js");
const fixture = join(__dirname, "../../fixtures/napi/worker-invocation-lock/worker.js");
const linux = process.platform === "linux";
const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const { configureCompiledApiEnvironment } = require("./compiled-api-environment.js");

for (const platform of ["darwin", "win32"]) {
  test(`compiled ${platform} setup selects the existing process-isolated invocation lock`, () => {
    const env = { NO_MISTAKES_TEST_NAPI_ADDON_PATH: "/compiled/addon.node" };
    configureCompiledApiEnvironment({ platform, env });
    assert.equal(
      env["CARGO_BIN_EXE_no-mistakes"],
      join("/compiled", platform === "win32" ? "no-mistakes.exe" : "no-mistakes"),
    );
    assert.equal(env.XDG_RUNTIME_DIR, undefined);

    const overridden = { ...env, NO_MISTAKES_TEST_CLI_PATH: "/explicit/cli" };
    configureCompiledApiEnvironment({ platform, env: overridden });
    assert.equal(overridden["CARGO_BIN_EXE_no-mistakes"], "/explicit/cli");
  });
}

test("mock addons and Linux setup preserve the incoming environment", () => {
  for (const platform of ["linux", "darwin", "win32"]) {
    for (const addon of [undefined, "/mock/addon.js", "/compiled/addon.node"]) {
      if (platform !== "linux" && addon?.endsWith(".node")) continue;
      const env = { NO_MISTAKES_TEST_NAPI_ADDON_PATH: addon, XDG_RUNTIME_DIR: "/runtime" };
      const before = { ...env };
      configureCompiledApiEnvironment({ platform, env });
      assert.deepEqual(env, before);
    }
  }
});

test(
  "compiled process isolation keeps native contention and independent CLI queries",
  { skip: !compiled },
  async () => {
    const child = fork(join(__dirname, "../../fixtures/napi/worker-invocation-lock/per-process.js"), {
      env: process.env,
      stdio: ["ignore", "ignore", "inherit", "ipc"],
    });
    const exited = once(child, "exit");
    try {
      const [message] = await Promise.race([
        once(child, "message"),
        exited.then(([code, signal]) => {
          throw new Error(`process isolation fixture exited before reporting: ${code}/${signal}`);
        }),
      ]);
      assert.deepEqual(message, { success: true });
      assert.deepEqual(await exited, [0, null]);
    } finally {
      child.kill();
    }
  },
);

function start(mode, initialize = "setup", environment = process.env) {
  const child = fork(fixture, [mode, initialize], {
    env: environment,
    stdio: ["ignore", "ignore", "inherit", "ipc"],
  });
  return { child, message: once(child, "message"), exited: once(child, "exit") };
}

test(
  "compiled setup assigns independent process runtime directories",
  { skip: !linux },
  async () => {
    const env = { ...process.env, NO_MISTAKES_TEST_NAPI_ADDON_PATH: "/unused/addon.node" };
    const first = start("environment", "setup", env);
    const second = start("environment", "setup", env);
    try {
      const [[a], [b]] = await Promise.all([first.message, second.message]);
      assert.notEqual(a.runtime, env.XDG_RUNTIME_DIR);
      assert.notEqual(a.runtime, b.runtime);
      assert.deepEqual(await first.exited, [0, null]);
      assert.deepEqual(await second.exited, [0, null]);
    } finally {
      first.child.kill();
      second.child.kill();
    }
  },
);

test(
  "compiled workers isolate invocation locks while inherited runtimes retain contention",
  { skip: !linux || !compiled, timeout: 20_000 },
  async () => {
    const holder = start("hold", "inherit");
    let independent;
    let inherited;
    try {
      const [held] = await holder.message;
      independent = start("query");
      const [free] = await independent.message;
      assert.equal(free.success, true);
      assert.notEqual(free.runtime, held.runtime);
      // A deliberate child of the same runtime must still observe the lock.
      inherited = start("query", "inherit", { ...process.env, XDG_RUNTIME_DIR: held.runtime });
      const [busy] = await inherited.message;
      assert.equal(busy.runtime, held.runtime);
      assert.match(busy.error, /another no-mistakes invocation is already running/);
      // Also exercise Rust environment access in the Vitest worker itself, not only child processes.
      const api = require("../../packages/no-mistakes/index.js");
      await assert.rejects(
        api.dependencies({
          root: join(__dirname, "../../fixtures/napi/real-addon-dependencies"),
          files: ["entry.ts"],
          failOnLock: true,
        }),
        /another no-mistakes invocation is already running/,
      );
      assert.deepEqual(await independent.exited, [0, null]);
      assert.deepEqual(await inherited.exited, [0, null]);
      holder.child.send("release");
      assert.deepEqual(await holder.exited, [0, null]);
    } finally {
      holder.child.kill();
      independent?.child.kill();
      inherited?.child.kill();
    }
  },
);
