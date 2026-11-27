const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { spawnSync } = require("node:child_process");
const path = require("node:path");
const { test } = require("node:test");
const { parse } = require("yaml");
const { rustRequired, triggersRust } = require("./rust-pr-scope.cjs");
const workflow = readFileSync(path.join(__dirname, "../workflows/ci.yml"), "utf8");
const job = (name) => workflow.split(`\n  ${name}:\n`)[1].split(/\n  [a-z][a-z-]*:\n/)[0];

for (const file of [
  "packages/eslint-plugin-no-mistakes/src/rules/example.js",
  "test-cases/eslint-snippets/fixture/example.ts",
  "plugins/no-mistakes-rules/skills/example/SKILL.md",
  "README.md",
  "CONTRIBUTING.md",
  "AGENTS.md",
  "evals/AGENTS.md",
  "evals/prompt.markdown",
  "packages/no-mistakes/README.md",
  "plugins/another-plugin/guide.MDX",
  "plugins/no-mistakes-rules/skills/react-rules/SKILL.md",
  "docs-sibling/example.md",
  ".cargo/README.md",
  "cspell.config.yaml",
  ".lycheeignore",
]) {
  test(`independent lint input skips Rust: ${file}`, () => assert.equal(triggersRust(file), false));
}
for (const file of [
  "crates/no-mistakes/src/lib.rs",
  "Cargo.toml",
  "Cargo.lock",
  "rust-toolchain.toml",
  ".cargo/config.toml",
  ".github/workflows/ci.yml",
  ".github/scripts/example.cjs",
  "packages/no-mistakes/index.js",
  "docs/rules/example.md",
  "docs/example.markdown",
  "docs/node-api.md",
  "plugins/no-mistakes/skills/no-mistakes/SKILL.md",
  "plugins/no-mistakes-rules/skills/github-actions-rules/SKILL.md",
  "crates/AGENTS.md",
  "docs/eslint-rules/example.md",
  "fixtures/check/example.md",
  "test-cases/eslint-snippets/fixture/example.md",
  "test-cases/eslint-snippets/Cargo.toml",
  "packages/eslint-plugin-no-mistakes/build.rs",
  "packages/eslint-plugin-no-mistakes/.cargo/config.toml",
  "test-cases/eslint-snippets/rust-toolchain",
  "unknown/input.js",
]) {
  test(`Rust or unknown input runs Rust: ${file}`, () => assert.equal(triggersRust(file), true));
}
test("the AST guard and the Rust readers share the selector's Markdown input folders", () => {
  const inputs = readFileSync(path.join(__dirname, "rust-markdown-inputs.txt"), "utf8")
    .trim()
    .split(/\r?\n/);
  const rule = parse(
    readFileSync(
      path.join(__dirname, "../../.ast-grep/rules/no-unscoped-repository-markdown-read.yml"),
      "utf8",
    ),
  );
  const allowed = rule.utils["unscoped-markdown"].all.find(({ not }) => not).not.all[0].regex;
  const prefixes = allowed.match(/\(\?:([^()]+)\)$/)[1].split("|");
  assert.deepEqual(prefixes.sort(), [...inputs].sort());
  for (const directory of inputs) assert.equal(triggersRust(`${directory}example.md`), true);
});

test("empty, mixed, deleted and renamed inputs stay conservative", () => {
  assert.equal(rustRequired([]), true);
  assert.equal(rustRequired([{ path: "README.md" }, { path: "cspell.config.yaml" }]), false);
  assert.equal(rustRequired([{ path: "README.md", status: "D" }]), false);
  assert.equal(rustRequired([{ path: "docs/eslint-rules/example.md", status: "D" }]), true);
  assert.equal(
    rustRequired([{ path: "docs/eslint-rules/example.md" }, { path: "Cargo.lock" }]),
    true,
  );
  const input = "D\0crates/no-mistakes/src/old.rs\0A\0docs/eslint-rules/old.md\0";
  const result = spawnSync(process.execPath, [path.join(__dirname, "rust-pr-scope.cjs")], {
    input,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout, "rust=true\n");
});
function gate(name, results, required) {
  const args =
    name === "tests-gate"
      ? [path.join(__dirname, "tests-gate.cjs")]
      : ["-e", job(name).match(/node -e '([^']+)'/)[1]];
  return spawnSync(process.execPath, args, {
    encoding: "utf8",
    env: {
      ...process.env,
      RESULTS: JSON.stringify(results),
      RUST_REQUIRED: required,
      MACOS_REQUIRED: "false",
      MACOS_EVENT: "pull_request",
      MACOS_REF: "refs/pull/1/merge",
    },
  });
}
for (const [name, allowed, forbidden] of [
  ["quality", "quality-rust", "quality-js"],
  ["tests-gate", "tests", "javascript-coverage"],
  ["tests-gate", "standalone-build", "linux-api-tests"],
  ["tests-gate", "standalone-shards", "packages"],
  ["tests-gate", "native-test-build", "changes"],
]) {
  test(`${name} allows only an intentional Rust skip (${allowed})`, () => {
    const skipped = { [allowed]: { result: "skipped" }, changes: { result: "success" } };
    assert.equal(gate(name, skipped, "false").status, 0);
    for (const required of ["true", ""]) assert.equal(gate(name, skipped, required).status, 1);
    for (const result of ["failure", "cancelled"])
      assert.equal(gate(name, { [allowed]: { result } }, "false").status, 1);
    assert.equal(gate(name, { [forbidden]: { result: "skipped" } }, "false").status, 1);
    assert.equal(gate(name, { changes: { result: "failure" } }, "false").status, 1);
  });
}
test("scope metadata is PR-only, fail-closed, and artifact merges skip with their producers", () => {
  assert.match(workflow, /rust:.*github.event_name != 'pull_request'.*outputs.rust != 'false'/);
  for (const name of ["quality-rust", "standalone-build", "rust-coverage-build", "docs-tests"])
    assert.match(job(name), /if: \$\{\{ needs.changes.outputs.rust == 'true' \}\}/);
  assert.match(
    job("tests"),
    /^    if: \$\{\{ !cancelled\(\) && needs.changes.outputs.rust == 'true' \}\}$/m,
  );
  assert.match(workflow, /node --test[^\n]*rust-pr-scope.test.cjs/);
  assert.match(job("tests-gate"), /name: Tests with coverage\n/);
  assert.match(job("tests-gate"), /if: \$\{\{ !cancelled\(\) \}\}/);
  assert.match(job("ast-analysis"), /name: AST analysis\n/);
  assert.doesNotMatch(job("ast-analysis"), /^    if:/m);
});
