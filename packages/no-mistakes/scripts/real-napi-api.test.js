const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { mkdtemp, mkdir, rm, stat } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join, resolve } = require("node:path");
const test = globalThis.test || require("node:test").test;

const repositoryRoot = join(__dirname, "..", "..", "..");
const fixtureRoot = join(repositoryRoot, "fixtures", "napi", "real-addon-dependencies");
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
  "compiled internal N-API rename preserves an existing destination",
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
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);
