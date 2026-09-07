const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const {
  buildMarkdown,
  commentMarker,
  comparableDurationSeconds,
  compileWorkloadFailed,
  findJobByName,
  findSuccessfulJob,
  formatDelta,
  formatDuration,
  isSuccessfulRun,
} = require("../../.github/scripts/report-native-job-timing.cjs");

test("formats durations and deltas", () => {
  assert.equal(formatDuration(0), "0s");
  assert.equal(formatDuration(45), "45s");
  assert.equal(formatDuration(565), "9m 25s");
  assert.equal(formatDelta(360, 565), "3m 25s faster");
  assert.equal(formatDelta(600, 565), "+35s slower");
  assert.equal(formatDelta(10, 10), "0s");
});

test("builds a before/after comment for the native job", () => {
  const markdown = buildMarkdown({
    jobName: "Windows x64",
    afterSha: "abcdef123456",
    beforeSha: "1234567abcdef",
    nowMs: Date.parse("2026-09-07T12:42:52Z"),
    afterJob: {
      started_at: "2026-09-07T12:33:27Z",
      completed_at: "2026-09-07T12:42:52Z",
      steps: [
        {
          name: "Run platform-specific Rust tests",
          started_at: "2026-09-07T12:34:42Z",
          completed_at: "2026-09-07T12:38:11Z",
        },
        {
          name: "Build native CLI and N-API addon",
          started_at: "2026-09-07T12:38:11Z",
          completed_at: "2026-09-07T12:42:41Z",
        },
      ],
    },
    beforeJob: {
      started_at: "2026-09-07T12:33:27Z",
      completed_at: "2026-09-07T12:42:52Z",
      steps: [
        {
          name: "Run platform-specific Rust tests",
          started_at: "2026-09-07T12:34:42Z",
          completed_at: "2026-09-07T12:38:11Z",
        },
        {
          name: "Build native CLI and N-API addon",
          started_at: "2026-09-07T12:38:11Z",
          completed_at: "2026-09-07T12:42:41Z",
        },
      ],
    },
  });

  assert.ok(markdown.includes(commentMarker("Windows x64")));
  assert.match(markdown, /9m 14s/);
  assert.match(markdown, /Run platform-specific Rust tests/);
  assert.match(markdown, /Build native CLI and N-API addon/);
});

test("baselines ignore unsuccessful runs and jobs", () => {
  assert.equal(isSuccessfulRun({ status: "completed", conclusion: "success" }), true);
  assert.equal(isSuccessfulRun({ status: "completed", conclusion: "failure" }), false);
  assert.equal(isSuccessfulRun({ status: "completed", conclusion: "cancelled" }), false);
  const jobs = [
    { name: "Native tests (Windows x64)", conclusion: null },
    { name: "Native tests (Windows x64)", conclusion: "success" },
  ];
  assert.equal(findJobByName(jobs, "Native tests (Windows x64)")?.conclusion, null);
  assert.equal(findSuccessfulJob(jobs, "Native tests (Windows x64)")?.conclusion, "success");
});

test("job totals stop at the last compile step, not post-job cleanup", () => {
  const nowMs = Date.parse("2026-09-07T12:50:00Z");
  const job = {
    started_at: "2026-09-07T12:33:27Z",
    completed_at: "2026-09-07T12:49:00Z",
    steps: [
      {
        name: "Build native CLI and N-API addon",
        started_at: "2026-09-07T12:38:11Z",
        completed_at: "2026-09-07T12:42:41Z",
      },
    ],
  };
  assert.equal(comparableDurationSeconds(job, nowMs), 554);
});

function successfulNativeJob(extraSteps = []) {
  return {
    started_at: "2026-09-07T12:33:27Z",
    steps: [
      {
        name: "Run platform-specific Rust tests",
        started_at: "2026-09-07T12:34:42Z",
        completed_at: "2026-09-07T12:38:11Z",
        conclusion: "success",
      },
      {
        name: "Build native CLI and N-API addon",
        started_at: "2026-09-07T12:38:11Z",
        completed_at: "2026-09-07T12:42:41Z",
        conclusion: "success",
      },
      {
        name: "Run native CLI smoke test",
        started_at: "2026-09-07T12:42:41Z",
        completed_at: "2026-09-07T12:42:50Z",
        conclusion: "success",
      },
      {
        name: "Run real N-API API test",
        started_at: "2026-09-07T12:42:50Z",
        completed_at: "2026-09-07T12:43:00Z",
        conclusion: "success",
      },
      ...extraSteps,
    ],
  };
}

test("failed compile or test steps suppress the performance delta", () => {
  const failed = {
    started_at: "2026-09-07T12:33:27Z",
    steps: [
      {
        name: "Build native CLI and N-API addon",
        started_at: "2026-09-07T12:38:11Z",
        completed_at: "2026-09-07T12:40:00Z",
        conclusion: "failure",
      },
    ],
  };
  assert.equal(compileWorkloadFailed(failed), true);
  const markdown = buildMarkdown({
    jobName: "Windows x64",
    afterJob: failed,
    beforeJob: failed,
    nowMs: Date.parse("2026-09-07T12:50:00Z"),
  });
  assert.match(markdown, /n\/a \(current run failed\)/);
});

test("skipped required workload steps suppress the performance delta", () => {
  const skipped = {
    started_at: "2026-09-07T12:33:27Z",
    steps: [
      {
        name: "Install pnpm dependencies",
        conclusion: "failure",
      },
      {
        name: "Run platform-specific Rust tests",
        conclusion: "skipped",
      },
      {
        name: "Build native CLI and N-API addon",
        conclusion: "skipped",
      },
      {
        name: "Run native CLI smoke test",
        conclusion: "skipped",
      },
      {
        name: "Run real N-API API test",
        conclusion: "skipped",
      },
    ],
  };
  assert.equal(compileWorkloadFailed(skipped), true);
  const markdown = buildMarkdown({
    jobName: "Windows x64",
    afterJob: skipped,
    beforeJob: successfulNativeJob(),
    nowMs: Date.parse("2026-09-07T12:50:00Z"),
  });
  assert.match(markdown, /n\/a \(current run failed\)/);
});

test("a skipped required step on the baseline suppresses the performance delta", () => {
  const incompleteBaseline = {
    started_at: "2026-09-07T12:33:27Z",
    conclusion: "success",
    steps: [
      {
        name: "Run platform-specific Rust tests",
        started_at: "2026-09-07T12:34:42Z",
        completed_at: "2026-09-07T12:38:11Z",
        conclusion: "success",
      },
      {
        name: "Build native CLI and N-API addon",
        conclusion: "skipped",
      },
      {
        name: "Run native CLI smoke test",
        conclusion: "skipped",
      },
      {
        name: "Run real N-API API test",
        conclusion: "skipped",
      },
    ],
  };
  assert.equal(compileWorkloadFailed(incompleteBaseline), true);
  const markdown = buildMarkdown({
    jobName: "Windows x64",
    afterJob: successfulNativeJob(),
    beforeJob: incompleteBaseline,
    nowMs: Date.parse("2026-09-07T12:50:00Z"),
  });
  assert.match(markdown, /n\/a \(base run incomplete\)/);
});

test("a skipped Defender step on an otherwise successful job stays comparable", () => {
  const macos = successfulNativeJob([
    {
      name: "Exclude workspace from Microsoft Defender",
      conclusion: "skipped",
    },
  ]);
  assert.equal(compileWorkloadFailed(macos), false);
  const markdown = buildMarkdown({
    jobName: "macOS ARM64",
    afterJob: macos,
    beforeJob: successfulNativeJob(),
    nowMs: Date.parse("2026-09-07T12:50:00Z"),
  });
  assert.doesNotMatch(markdown, /n\/a \(current run failed\)/);
  assert.match(markdown, /\*\*0s\*\*/);
});

test("exact-SHA baseline lookup is restricted to push runs", () => {
  const source = fs.readFileSync(
    path.join(__dirname, "../../.github/scripts/report-native-job-timing.cjs"),
    "utf8",
  );
  const shaLookup = source.slice(source.indexOf("const shaRuns"), source.indexOf("const branchRuns"));
  assert.match(shaLookup, /"--event",\s*"push"/);
});
