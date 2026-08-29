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
