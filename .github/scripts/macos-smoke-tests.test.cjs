const assert = require("node:assert/strict");
const { existsSync, readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");
const { cargoCommands, vitestArgs } = require("./macos-smoke.cjs");
const { api, cliFixture, rust } = require("./macos-smoke-tests.cjs");

const root = path.resolve(__dirname, "../..");
const read = (file) => readFileSync(path.join(root, file), "utf8");

test("every listed Rust target, file and test name still exists", () => {
  const manifest = read("crates/no-mistakes/Cargo.toml");
  assert.ok(rust.length > 0);
  for (const { selector, guards } of rust) {
    assert.ok(guards.length > 0);
    if (selector[0] === "--test")
      assert.match(manifest, new RegExp(`\\[\\[test\\]\\]\\nname = "${selector[1]}"`));
    for (const { file, text } of guards) {
      assert.ok(existsSync(path.join(root, file)), file);
      assert.ok(read(file).includes(text), `${file} must contain ${text}`);
    }
  }
  assert.ok(cargoCommands().every((args) => args.includes("--locked")));
});

test("every listed API test file and test name still exists", () => {
  assert.ok(api.length > 0);
  const seen = new Set();
  for (const { file, names } of api) {
    assert.ok(existsSync(path.join(root, file)), file);
    const source = read(file);
    assert.ok(names.length > 0);
    for (const name of names) {
      assert.ok(source.includes(`"${name}"`), `${file} must contain test "${name}"`);
      assert.ok(!seen.has(name), `duplicate ${name}`);
      seen.add(name);
    }
  }
  assert.match(vitestArgs().join(" "), /--testNamePattern=\^\(\?:/);
});

test("the listed CLI fixture is saved in the repository", () => {
  assert.ok(existsSync(path.join(root, cliFixture, ".no-mistakes.yml")));
});

test("the macOS smoke job runs on pull requests and drives the checked-in list", () => {
  const body = workflowJob(read(".github/workflows/ci.yml"), "macos-smoke");
  assert.match(body, /needs\.changes\.outputs\.macos-smoke == 'true'/);
  assert.match(body, /runs-on: macos-15/);
  assert.doesNotMatch(body, /windows/i);
  for (const mode of ["rust", "cli", "api"])
    assert.match(body, new RegExp(`macos-smoke\\.cjs ${mode}`));
  assert.doesNotMatch(body, /pull_request_target|secrets\./);
});
