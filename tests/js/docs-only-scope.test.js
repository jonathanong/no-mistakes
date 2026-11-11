const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const repoRoot = join(__dirname, "..", "..");
const selector = join(repoRoot, ".github", "scripts", "docs-only-scope.cjs");
const { docsOnly } = require(selector);
const modified = (paths) => paths.map((path) => ({ status: "M", path }));
const docsOnlyFor = (path) => docsOnly(modified([path]));

function run(input, args = []) {
  return spawnSync(process.execPath, [selector, ...args], { input, encoding: "utf8" });
}

test("markdown the docs tests read is docs-only", () => {
  for (const path of [
    "README.md",
    "CONTRIBUTING.md",
    "AGENTS.md",
    "docs/why.md",
    "docs/README.md",
    "docs/rules/postgres-no-offset.md",
    "docs/cli/symbols.md",
    "docs/configuration/README.md",
    "docs/performance/call-site-assembly.md",
    ".github/pull_request_template.md",
  ]) {
    assert.equal(docsOnlyFor(path), true, path);
  }
  assert.equal(docsOnly(modified(["README.md", "docs/why.md"])), true);
});

test("markdown that only full test jobs read runs every test job", () => {
  for (const path of [
    "docs/eslint-rules/test-no-skips.md",
    "docs/eslint-plugin.md",
    "docs/cli/tests-audit.md",
    "plugins/no-mistakes/skills/no-mistakes/SKILL.md",
    "packages/no-mistakes/README.md",
    "crates/no-mistakes/README.md",
    "crates/AGENTS.md",
    "fixtures/rules/disable-directive-audit/bom.md",
    "test-cases/markdown/guide.md",
    "tests/js/notes.md",
    "evals/01-dead-in-production/prompt.md",
  ]) {
    assert.equal(docsOnlyFor(path), false, path);
  }
});

test("any non-markdown change runs every test job", () => {
  assert.equal(docsOnly(modified(["README.md", ".github/workflows/ci.yml"])), false);
  assert.equal(docsOnlyFor(".github/scripts/docs-only-scope.cjs"), false);
  assert.equal(docsOnlyFor("docs/performance/manifest.json"), false);
  assert.equal(docsOnly([]), false);
});

test("CLI reads NUL-separated name-status pairs and fails closed without a base", () => {
  assert.equal(run("M\0README.md\0D\0docs/why.md\0").stdout, "docs-only=true\n");
  assert.equal(run("M\0README.md\0M\0Cargo.toml\0").stdout, "docs-only=false\n");
  assert.equal(run("", ["--all"]).stdout, "docs-only=false\n");
  assert.notEqual(run("R100\0a.md\0").status, 0);
});

test("docs-only pull requests still run the docs contract suite", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
  assert.match(workflow, /--test suite_contracts/);
  assert.match(workflow, /docs-only: \$\{\{ github\.event_name == 'pull_request' &&/);
});
