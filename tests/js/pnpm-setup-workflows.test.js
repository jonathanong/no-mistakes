const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const repoRoot = join(__dirname, "..", "..");
const ciWorkflow = readFileSync(join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
const releaseWorkflow = readFileSync(
  join(repoRoot, ".github", "workflows", "release.yml"),
  "utf8",
);

test("workflows use the pnpm 11 setup action with immutable refs", () => {
  for (const workflow of [ciWorkflow, releaseWorkflow]) {
    assert.doesNotMatch(workflow, /pnpm\/action-setup@/u);

    const setupRefs = [...workflow.matchAll(/uses: pnpm\/setup@(\S+)/gu)].map(
      ([, setupRef]) => setupRef,
    );
    assert.ok(setupRefs.length > 0);
    for (const setupRef of setupRefs) {
      assert.match(setupRef, /^[0-9a-f]{40}$/u);
    }
  }
});

test("CI setup installs Node 24 without an extra setup-node action", () => {
  assert.match(ciWorkflow, /runtime: node@24/u);
  assert.match(ciWorkflow, /cache: true/u);
  assert.match(ciWorkflow, /install: false/u);
  assert.doesNotMatch(ciWorkflow, /actions\/setup-node@/u);
});

test("release setup preserves npm registry configuration", () => {
  assert.match(releaseWorkflow, /uses: actions\/setup-node@[0-9a-f]{40} # v\d+/u);
  assert.match(releaseWorkflow, /registry-url: https:\/\/registry\.npmjs\.org/u);
  assert.match(releaseWorkflow, /uses: pnpm\/setup@[0-9a-f]{40} # v\d+/u);
  assert.match(releaseWorkflow, /install: false/u);
});
