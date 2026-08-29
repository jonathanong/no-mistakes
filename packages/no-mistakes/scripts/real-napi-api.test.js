const assert = require("node:assert/strict");
const { fork } = require("node:child_process");
const { once } = require("node:events");
const { readFileSync } = require("node:fs");
const { mkdtemp, mkdir, rm, stat } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join, resolve } = require("node:path");
const { setTimeout: delay } = require("node:timers/promises");
const { Worker } = require("node:worker_threads");
const test = globalThis.test || require("node:test").test;

const repositoryRoot = join(__dirname, "..", "..", "..");
const fixtureRoot = join(repositoryRoot, "fixtures", "napi", "real-addon-dependencies");
const lockHolderFixture = join(
  repositoryRoot,
  "fixtures",
  "napi",
  "planning-artifact-lock",
  "hold.js",
);
const workerLockHolderFixture = join(
  repositoryRoot,
  "fixtures",
  "napi",
  "planning-artifact-lock",
  "worker.js",
);
const expectedReport = JSON.parse(readFileSync(join(fixtureRoot, "expected.json"), "utf8"));
const addonPath = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "compiled async N-API dependencies API matches the CLI fixture contract",
  { skip: !addonPath },
  async () => {
    assert.equal(resolve(addonPath), addonPath);
    assert.match(addonPath, /\.node$/);

    const api = require("../index.js");
    const pendingReport = api.dependencies({
      root: fixtureRoot,
      files: ["entry.ts"],
      relationships: ["import"],
    });

    assert.equal(typeof pendingReport.then, "function");
    assert.deepEqual(await pendingReport, expectedReport);
  },
);

test(
  "compiled internal N-API lock serializes separate Node processes",
  { skip: !addonPath },
  async () => {
    const directory = await mkdtemp(join(tmpdir(), "no-mistakes-napi-lock-"));
    const lockPath = join(directory, "artifact.lock");
    const native = require(addonPath);
    const child = fork(lockHolderFixture, [lockPath], {
      env: { ...process.env, NO_MISTAKES_TEST_NAPI_ADDON_PATH: addonPath },
      stdio: ["ignore", "ignore", "inherit", "ipc"],
    });
    let parentToken;
    try {
      const [message] = await once(child, "message");
      assert.equal(message, "locked");
      await assert.rejects(
        native.acquirePlanningArtifactLock(lockPath),
        /planning artifact lock is busy/,
      );

      const exited = once(child, "exit");
      child.send("release");
      assert.deepEqual(await exited, [0, null]);
      parentToken = await native.acquirePlanningArtifactLock(lockPath);
      await native.releasePlanningArtifactLock(parentToken);
      parentToken = undefined;
    } finally {
      if (parentToken !== undefined) await native.releasePlanningArtifactLock(parentToken);
      child.kill();
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "compiled internal N-API lock is released when a worker terminates",
  { skip: !addonPath },
  async () => {
    const directory = await mkdtemp(join(tmpdir(), "no-mistakes-napi-worker-lock-"));
    const lockPath = join(directory, "artifact.lock");
    const worker = new Worker(workerLockHolderFixture, {
      workerData: { addonPath, lockPath },
    });
    let child;
    try {
      const [message] = await once(worker, "message");
      assert.equal(message, "locked");
      assert.equal(await worker.terminate(), 1);

      child = fork(lockHolderFixture, [lockPath], {
        env: { ...process.env, NO_MISTAKES_TEST_NAPI_ADDON_PATH: addonPath },
        stdio: ["ignore", "ignore", "inherit", "ipc"],
      });
      const [childMessage] = await Promise.race([
        once(child, "message"),
        delay(2_000, undefined, { ref: false }).then(() => {
          throw new Error("terminated worker retained its planning artifact lock");
        }),
      ]);
      assert.equal(childMessage, "locked");
      const exited = once(child, "exit");
      child.send("release");
      assert.deepEqual(await exited, [0, null]);
      child = undefined;
    } finally {
      await worker.terminate();
      child?.kill();
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "compiled internal N-API rename preserves an existing destination and claims a vacant one",
  { skip: !addonPath },
  async () => {
    const directory = await mkdtemp(join(tmpdir(), "no-mistakes-napi-rename-"));
    const source = join(directory, "source");
    const destination = join(directory, "destination");
    try {
      await mkdir(source);
      await mkdir(destination);
      const native = require(addonPath);
      assert.equal(await native.renameNoReplace(source, destination), false);
      assert.equal((await stat(source)).isDirectory(), true);
      assert.equal((await stat(destination)).isDirectory(), true);
      await rm(destination, { recursive: true });
      assert.equal(await native.renameNoReplace(source, destination), true);
      await assert.rejects(stat(source), { code: "ENOENT" });
      assert.equal((await stat(destination)).isDirectory(), true);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);
