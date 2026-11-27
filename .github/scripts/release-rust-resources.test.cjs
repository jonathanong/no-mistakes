const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const YAML = require("yaml");
const cases = require("./fixtures/release-rust-resources.json");

const root = path.resolve(__dirname, "../..");
const source = readFileSync(path.join(root, ".github/workflows/release.yml"), "utf8");
const tools = "$RUNNER_TEMP/validation-tools/.github/scripts/";
const sampler = `bash "${tools}with-resource-samples.sh" `;
const commands = {
  "Run Clippy": "cargo clippy --locked --workspace --all-targets --all-features -- -D warnings",
  "Build Rust tests": `bash "${tools}rust-validation.sh" build`,
  "Run Rust tests (unit)": `bash "${tools}rust-validation.sh" test-unit`,
  "Run Rust tests (integration)": `bash "${tools}rust-validation.sh" test-integration`,
  "Run Rust doctests": `bash "${tools}rust-validation.sh" doc`,
  "Check Cargo packages": "cargo publish --dry-run --locked --workspace",
};
const settings = {
  CARGO_BUILD_JOBS: "1",
  CARGO_INCREMENTAL: "0",
  CARGO_PROFILE_DEV_CODEGEN_UNITS: "256",
  CARGO_PROFILE_TEST_CODEGEN_UNITS: "256",
  CARGO_PROFILE_DEV_DEBUG: "0",
  CARGO_PROFILE_TEST_DEBUG: "0",
  RUST_TEST_THREADS: "4",
  RAYON_NUM_THREADS: "1",
};

function violations(job) {
  const problems = [];
  for (const [key, value] of Object.entries(settings)) {
    if (job.env?.[key] !== value) problems.push(key);
  }
  for (const [name, command] of Object.entries(commands)) {
    const matches = job.steps.filter((step) => step.name === name);
    if (matches.length !== 1 || matches[0].run?.trim() !== sampler + command) problems.push(name);
  }
  const budget = job.steps.find((step) => step.name === "Record Rust resource budget")?.run ?? "";
  for (const key of [...Object.keys(settings), "RUSTFLAGS"]) {
    if (!budget.includes(`"$${key}"`)) problems.push(`budget:${key}`);
  }
  for (const command of ["rustc -vV", "free -m"]) {
    if (!budget.split("\n").includes(command)) problems.push(`budget:${command}`);
  }
  return problems;
}

function applyMutation(job, change) {
  if (change.env) {
    for (const [key, value] of Object.entries(change.env)) {
      if (value === null) delete job.env[key];
      else job.env[key] = value;
    }
  }
  if (change.step) {
    const step = job.steps.find((entry) => entry.name === change.step);
    step.run = step.run.replace(change.replace, change.with);
  }
}

test("release validation caps compiler and runtime workers and samples every heavy phase", () => {
  const job = YAML.parse(source).jobs["validate-rust"];
  assert.deepEqual(violations(job), []);
});

for (const scenario of cases) {
  test(`release resource controls reject ${scenario.name}`, () => {
    const job = YAML.parse(source).jobs["validate-rust"];
    applyMutation(job, scenario.change);
    assert.deepEqual(violations(job), scenario.expected);
  });
}
