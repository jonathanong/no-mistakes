const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const { platformTarget } = require("../../packages/no-mistakes/scripts/install/platform");

const repoRoot = join(__dirname, "..", "..");

function releaseMatrixTargets(source) {
  const match = source.match(/^ {6}matrix:\n {8}target:\n((?: {10}- [^\n]+\n)+)/m);
  assert.ok(match, "release workflow must define the build target matrix");
  return match[1]
    .trim()
    .split("\n")
    .map((line) => line.trim().replace(/^- /, ""));
}

test("every installer platform target has a release build", () => {
  const releaseWorkflow = readFileSync(
    join(repoRoot, ".github", "workflows", "release.yml"),
    "utf8",
  );
  const releaseTargets = releaseMatrixTargets(releaseWorkflow);
  const installerTargets = [
    platformTarget("darwin", "x64"),
    platformTarget("darwin", "arm64"),
    platformTarget("win32", "x64"),
    platformTarget("linux", "x64", {
      getReport: () => ({ header: { glibcVersionRuntime: "2.35" } }),
    }),
    platformTarget("linux", "arm64", {
      getReport: () => ({ header: { glibcVersionRuntime: "2.35" } }),
    }),
  ];

  for (const target of installerTargets) {
    assert.ok(releaseTargets.includes(target), `release workflow does not build ${target}`);
  }
});

test("release builds leave enough time for Intel macOS binary and N-API compilation", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  const buildJob = workflow.match(/^ {2}build:[\s\S]*?(?=^ {2}publish:)/m);
  assert.ok(buildJob, "release workflow must define the build job");
  const jobTimeout = buildJob[0].match(/^ {4}timeout-minutes: (\d+)$/m);
  const buildTimeout = buildJob[0].match(/^ {6}- name: Build binary\n {8}timeout-minutes: (\d+)$/m);
  assert.ok(jobTimeout, "release build job must define a timeout");
  assert.ok(buildTimeout, "release binary step must define a timeout");
  assert.ok(Number(jobTimeout[1]) >= 70, "release build job timeout must be at least 70 minutes");
  assert.ok(
    Number(buildTimeout[1]) >= 60,
    "release binary step timeout must be at least 60 minutes",
  );
  assert.ok(
    Number(jobTimeout[1]) - Number(buildTimeout[1]) >= 10,
    "release build job must leave at least 10 minutes after binary compilation",
  );
});

test("native CI jobs run only platform-specific Rust tests", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
  const nativeJob = workflow.match(/^ {2}native-tests:[\s\S]*?(?=^ {2}[a-z])/m);
  assert.ok(nativeJob, "ci.yml must define native-tests");
  const body = nativeJob[0];

  assert.doesNotMatch(
    body,
    /cargo test\b[^\r\n]*--workspace\b/,
    "native jobs must not compile or run the Linux full suite",
  );
  assert.match(
    body,
    /cargo test --locked -p no-mistakes --lib --all-features "\$filter"/,
    "native jobs must compile only the no-mistakes lib tests",
  );
  assert.match(
    body,
    /invocation::tests::command_output_resumes_child_after_job_assignment/,
    "Windows must run the Job Object regression in isolation",
  );
  assert.match(body, /rust_test: ["']invocation::["']/);
  assert.match(body, /Run native CLI smoke test/);
  assert.match(body, /real-napi-api\.test\.js/);
  assert.match(
    workflow,
    /cargo test --workspace --all-features/,
    "Linux coverage keeps the full-suite spelling the native guard must reject",
  );
});
