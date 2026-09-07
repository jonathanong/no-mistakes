const assert = require("node:assert/strict");
const {
  buildMarkdown,
  commentMarker,
  findJob,
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
  assert.match(markdown, /9m 25s/);
  assert.match(markdown, /Run platform-specific Rust tests/);
  assert.match(markdown, /Build native CLI and N-API addon/);
});

test("baselines ignore unsuccessful runs and jobs", () => {
  assert.equal(isSuccessfulRun({ status: "completed", conclusion: "success" }), true);
  assert.equal(isSuccessfulRun({ status: "completed", conclusion: "failure" }), false);
  assert.equal(isSuccessfulRun({ status: "completed", conclusion: "cancelled" }), false);
  assert.equal(
    findJob(
      [
        { name: "Native tests (Windows x64)", conclusion: "cancelled" },
        { name: "Native tests (Windows x64)", conclusion: "success" },
      ],
      "Native tests (Windows x64)",
    )?.conclusion,
    "success",
  );
});
