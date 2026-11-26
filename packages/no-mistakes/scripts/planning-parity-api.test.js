const assert = require("node:assert/strict");
const { execFile, execFileSync } = require("node:child_process");
const { cp, mkdtemp, readFile, rm } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { promisify } = require("node:util");
const test = globalThis.test || require("node:test").test;

const addonPath = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;
const compiled = addonPath?.endsWith(".node");
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

test(
  "Node planning reads a saved manifest and includes a comment only on request",
  { skip: !compiled },
  async () => {
    const { impactedChecks, testsPlan, testsComment } = require("../index.js");
    const changedFilesFile = join(root, "reports/changed-files.txt");
    const expected = JSON.parse(await readFile(join(fixture, "changed-files.snap"), "utf8"));
    const options = { root, changedFilesFile, framework: "vitest" };
    const plan = await testsPlan(options);
    const commented = await testsPlan({ ...options, includeComment: true });
    assert.deepEqual((await impactedChecks({ root, changedFilesFile })).changed_files, expected);
    assert.deepEqual(plan.changedFiles, expected);
    assert.deepEqual(
      plan.selectedTests.map(({ testFile }) => testFile),
      ["tests/example.test.ts"],
    );
    assert.equal(Object.hasOwn(plan, "comment"), false);
    assert.equal(commented.comment, await testsComment({ planJson: plan }));
    assert.match(commented.comment, /tests\/example\.test\.ts/);
  },
);

test(
  "legacy saved plans render and Node collects diagnostics around text outputs",
  { skip: !compiled },
  async () => {
    const api = require("../index.js");
    const plan = join(fixture, "legacy-plan.json");
    const graph = JSON.parse(await readFile(join(fixture, "legacy-graph.snap"), "utf8"));
    assert.deepEqual(await api.testsGraph({ plan }), graph);
    for (const operation of ["testsComment", "testsGraphMermaid"]) {
      const expected = await api[operation]({ plan });
      const observed = await api.collectDiagnostics(operation, { plan });
      assert.equal(typeof expected, "string");
      assert.equal(observed.result, expected);
      assert.ok(observed.diagnostics.timings.some(({ label }) => label === "invocation.dispatch"));
    }
  },
);

test(
  "Node text renderers preserve the machine-readable invocation lock error",
  { skip: !compiled },
  async () => {
    const { stdout } = await execute(
      process.execPath,
      [join(fixture, "text-lock.cjs"), join(fixture, "legacy-plan.json")],
      { env: { ...process.env, "CARGO_BIN_EXE_no-mistakes": "planning-parity" } },
    );
    assert.equal(stdout.trim(), "2 structured lock errors");
  },
);
