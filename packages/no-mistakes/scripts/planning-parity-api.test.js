const assert = require("node:assert/strict");
const { execFile, execFileSync } = require("node:child_process");
const { cp, mkdtemp, readFile, rm } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { promisify } = require("node:util");
const test = globalThis.test || require("node:test").test;

const compiled = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");
const repository = join(__dirname, "../../..");
const root = join(repository, "test-cases/tests-plan-changes");
const fixture = join(repository, "fixtures/node-api/planning-parity");
const diffFile = join(root, "reports/planning.patch");
const execute = promisify(execFile);

test(
  "impactedChecks accepts inline and file inputs from the same saved patch",
  { skip: !compiled },
  async () => {
    const { impactedChecks } = require("../index.js");
    const expected = JSON.parse(await readFile(join(fixture, "changed-files.snap"), "utf8"));
    const fromFile = await impactedChecks({ root, diffFile });
    const fromText = await impactedChecks({ root, diff: await readFile(diffFile, "utf8") });
    assert.deepEqual(fromFile.changed_files, expected);
    assert.deepEqual(fromFile, fromText);
  },
);

test(
  "Node planners reject head without base and accept an explicit revision pair",
  { skip: !compiled },
  async () => {
    const { impactedChecks, testsPlan } = require("../index.js");
    const directory = await mkdtemp(join(tmpdir(), "no-mistakes-planning-parity-"));
    try {
      await cp(root, directory, { recursive: true });
      const git = (...args) => execFileSync("git", ["-C", directory, ...args]);
      git("init", "-q");
      git("add", ".");
      git(
        "-c",
        "user.name=Example",
        "-c",
        "user.email=example@example.invalid",
        "-c",
        "commit.gpgsign=false",
        "commit",
        "-qm",
        "fixture",
      );
      const planning = { root: directory, framework: "vitest", head: "HEAD" };
      const impact = { root: directory, head: "HEAD" };
      await assert.rejects(testsPlan(planning), /head requires base/);
      await assert.rejects(impactedChecks(impact), /head requires base/);
      assert.deepEqual((await testsPlan({ ...planning, base: "HEAD" })).changedFiles, []);
      assert.deepEqual((await impactedChecks({ ...impact, base: "HEAD" })).changed_files, []);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  "impactedChecks rejects every conflicting patch source before I/O",
  { skip: !compiled },
  async () => {
    const { stdout } = await execute(
      process.execPath,
      [join(fixture, "conflicting-inputs.cjs"), root],
      { timeout: 5_000 },
    );
    assert.equal(stdout.trim(), "6 conflicts rejected");
  },
);
