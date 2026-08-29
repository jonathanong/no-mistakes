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

test("native CI jobs run only platform-specific Rust tests", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
  const nativeJob = workflow.match(/^ {2}native-tests:[\s\S]*?(?=^ {2}[a-z])/m);
  assert.ok(nativeJob, "ci.yml must define native-tests");
  const body = nativeJob[0];

  assert.doesNotMatch(
    body,
    /cargo test --locked --workspace/,
    "native jobs must not compile or run the Linux full suite",
  );
  assert.match(
    body,
    /cargo test --locked -p no-mistakes --lib --all-features "invocation::"/,
    "native jobs must filter to OS-specific invocation tests",
  );
  assert.match(body, /Run native CLI smoke test/);
  assert.match(body, /real-napi-api\.test\.js/);
});
