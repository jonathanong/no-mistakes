const assert = require("node:assert/strict");
const { fork } = require("node:child_process");
const { once } = require("node:events");
const { readFileSync } = require("node:fs");
const { mkdtemp, mkdir, rm, stat } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join, resolve } = require("node:path");
const { setTimeout: delay } = require("node:timers/promises");
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
      assert.deepEqual(await once(child, "message"), ["locked"]);
      let parentAcquired = false;
      const pendingToken = native.acquirePlanningArtifactLock(lockPath).then((token) => {
        parentAcquired = true;
        return token;
      });
      await delay(50);
      assert.equal(parentAcquired, false);

      const exited = once(child, "exit");
      child.send("release");
      assert.deepEqual(await exited, [0, null]);
      parentToken = await pendingToken;
      assert.equal(parentAcquired, true);
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
