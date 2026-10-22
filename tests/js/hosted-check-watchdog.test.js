const assert = require("node:assert/strict");
const { closeSync, existsSync, mkdtempSync, openSync, readFileSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { budgetReport } = require("../../.github/scripts/hosted-check-budget.cjs");
const { parseTime } = require("../../.github/scripts/hosted-check-metrics.cjs");
const {
  processTree,
  readLiveProcesses,
  timedCommand,
  SNAPSHOT_BEFORE_DEADLINE_SECONDS,
} = require("../../.github/scripts/hosted-check-watchdog.cjs");
const { fixture, samplePair } = require("./hosted-check-fixtures.cjs");

test("saved all-timeout cohorts fail as candidates under frozen budgets", () => {
  const groups = new Map();
  const samples = fixture("all-timeout-cohort.json");
  const commits = [
    "22edf4bfe0a782ac0d030c916dedfd6f8acb499c",
    "92d0a2061bde8d260aa0b86a2d71110ece1a43ca",
  ];
  for (const commit of commits)
    for (const jobs of ["1", "4", "auto"])
      for (const variant of ["baseline", "candidate"])
        groups.set(`${commit}/${jobs}/${variant}`, samplePair(samples, variant, commit, jobs));
  const budgets = JSON.parse(
    readFileSync(join(__dirname, "../../.github/scripts/hosted-check-budgets.json"), "utf8"),
  );
  const report = budgetReport(groups, budgets);
  assert.equal(report.failures.length, 6);
  assert.ok(report.failures.every((failure) => failure.includes("Candidate did not complete")));
  assert.deepEqual(report.proposed, {});
  assert.deepEqual(report.comparisons, {});
});

test("live snapshot follows a detached analyzer group and excludes unrelated processes", () => {
  const saved = readFileSync(
    join(__dirname, "../../fixtures/hosted-check/live-process-tree.ps"),
    "utf8",
  );
  const native = "/tmp/fixture/node_modules/no-mistakes-linux-x64-gnu/bin/no-mistakes";
  const tree = processTree(saved, 50, native);
  assert.match(tree, / 53 .*no-mistakes-linux-x64-gnu\/bin\/no-mistakes check/);
  assert.match(tree, /^55 1 55 .*no-mistakes-linux-x64-gnu\/bin\/no-mistakes check$/m);
  assert.match(tree, /^52 51 50 .*pnpm exec no-mistakes check$/m);
  assert.doesNotMatch(tree, /unrelated\.cjs|\/tmp\/other\//);
  assert.equal(SNAPSHOT_BEFORE_DEADLINE_SECONDS, 60);
});

// The hosted-check command uses GNU time and timeout on Linux runners.
(process.platform === "linux" ? test : test.skip)(
  "watchdog reads one live timed-out process tree and none for a completed check",
  async () => {
    const output = mkdtempSync(join(tmpdir(), "hosted-check-live-"));
    const script = join(__dirname, "../../fixtures/hosted-check/live-snapshot-child.cjs");
    try {
      let reads = 0;
      const livePath = join(output, "live.txt");
      const timePath = join(output, "live.time");
      const stdoutPath = join(output, "live.stdout");
      const stdout = openSync(stdoutPath, "w");
      let live;
      try {
        live = await timedCommand(
          "/usr/bin/time",
          [
            "-o",
            timePath,
            "-f",
            "elapsed=%e\\nuser=%U\\nsystem=%S\\nrss_kib=%M\\nexit=%x",
            "timeout",
            "--foreground",
            "--signal=TERM",
            "--kill-after=1s",
            "0.8s",
            process.execPath,
            script,
            "--hang",
          ],
          {
            cwd: output,
            stdio: ["ignore", stdout, "ignore"],
            snapshotPath: livePath,
            deadlineSeconds: 1,
            snapshotDelayMs: 80,
            readProcessTable: (cwd) => {
              reads += 1;
              return readLiveProcesses(cwd);
            },
          },
        );
      } finally {
        closeSync(stdout);
      }
      assert.equal(live.status, 124);
      assert.equal(parseTime(readFileSync(timePath, "utf8"), live.status).exit, 124);
      assert.match(readFileSync(stdoutPath, "utf8"), /fixture analyzer started/);
      assert.equal(reads, 1);
      assert.match(readFileSync(livePath, "utf8"), /live-snapshot-child\.cjs/);

      const fastPath = join(output, "fast.txt");
      const fast = await timedCommand(process.execPath, [script, "--fast"], {
        cwd: output,
        stdio: "ignore",
        snapshotPath: fastPath,
        deadlineSeconds: 1,
        snapshotDelayMs: 300,
        readProcessTable: () => {
          reads += 1;
          return "";
        },
      });
      assert.equal(fast.status, 0);
      assert.equal(reads, 1);
      assert.equal(existsSync(fastPath), false);
    } finally {
      rmSync(output, { recursive: true, force: true });
    }
  },
);

test("snapshot failure is retained without hiding the command exit", async () => {
  const output = mkdtempSync(join(tmpdir(), "hosted-check-snapshot-failure-"));
  const script = join(__dirname, "../../fixtures/hosted-check/live-snapshot-child.cjs");
  try {
    assert.throws(() => readLiveProcesses(output, "missing-hosted-check-ps"), /ps failed/);
    const path = join(output, "failed.txt");
    const result = await timedCommand(process.execPath, [script], {
      cwd: output,
      stdio: "ignore",
      snapshotPath: path,
      deadlineSeconds: 1,
      snapshotDelayMs: 80,
      readProcessTable: () => {
        throw new Error("ps unavailable");
      },
    });
    assert.equal(result.status, 0);
    assert.match(readFileSync(path, "utf8"), /Live process snapshot failed: Error: ps unavailable/);
    const unwritable = await timedCommand(process.execPath, [script], {
      cwd: output,
      stdio: "ignore",
      snapshotPath: join(output, "missing", "unwritable.txt"),
      deadlineSeconds: 1,
      snapshotDelayMs: 80,
      readProcessTable: () => {
        throw new Error("ps unavailable");
      },
    });
    assert.equal(unwritable.status, 0);
  } finally {
    rmSync(output, { recursive: true, force: true });
  }
});
