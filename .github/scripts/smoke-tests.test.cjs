const assert = require("node:assert/strict");
const { existsSync, readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");
const {
  assertCargoTestsRan,
  assertCliResult,
  buildCommand,
  cargoCommands,
  vitestArgs,
} = require("./smoke.cjs");
const { cliFixture, napiFixture, platforms } = require("./smoke-tests.cjs");

const root = path.resolve(__dirname, "../..");
const read = (file) => readFileSync(path.join(root, file), "utf8");
const systems = Object.entries(platforms);

test("every OS lists Rust and API tests", () => {
  assert.deepEqual(Object.keys(platforms).sort(), ["macos", "windows"]);
  assert.throws(() => cargoCommands("linux"), /unsupported smoke OS/);
  for (const [, { label, rust, api }] of systems) {
    assert.ok(label);
    assert.ok(rust.length > 0);
    assert.ok(api.length > 0);
  }
});

test("every listed Rust target, file and test name still exists", () => {
  const manifest = read("crates/no-mistakes/Cargo.toml");
  for (const [os, { rust }] of systems) {
    for (const { selector, guards } of rust) {
      assert.ok(guards.length > 0, os);
      if (selector[0] === "--test")
        assert.match(manifest, new RegExp(`\\[\\[test\\]\\]\\nname = "${selector[1]}"`));
      for (const { file, text } of guards) {
        assert.ok(existsSync(path.join(root, file)), file);
        assert.ok(read(file).includes(text), `${file} must contain ${text}`);
      }
    }
    assert.ok(cargoCommands(os).every((args) => args.includes("--locked")));
  }
});

test("every listed API test file and test name still exists", () => {
  for (const [os, { api }] of systems) {
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
    assert.match(vitestArgs(os).join(" "), /--testNamePattern=\^\(\?:/);
  }
});

test("the listed CLI and N-API fixtures are saved in the repository", () => {
  assert.ok(existsSync(path.join(root, cliFixture, ".no-mistakes.yml")));
  assert.ok(existsSync(path.join(root, napiFixture, "entry.ts")));
});

test("a Cargo filter that runs zero tests fails the smoke run", () => {
  const [, blockedCheck] = cargoCommands("macos");
  for (const output of ["running 0 tests\n\ntest result: ok. 0 passed\n", ""]) {
    assert.throws(() => assertCargoTestsRan(blockedCheck, output), /ran zero tests/);
  }
  assert.doesNotThrow(() =>
    assertCargoTestsRan(blockedCheck, "running 0 tests\r\nrunning 3 tests\r\n"),
  );
  assert.doesNotThrow(() => assertCargoTestsRan(blockedCheck, "running 1 test\n"));
});

test("CLI smoke requires the expected finding and the findings exit status", () => {
  const { finding, clean } = JSON.parse(read(".github/scripts/fixtures/smoke-cli-check.json"));
  const stdout = JSON.stringify(finding);
  assert.doesNotThrow(() => assertCliResult({ status: 1, stdout, stderr: "" }));
  assert.throws(
    () => assertCliResult({ status: 0, stdout: JSON.stringify(clean), stderr: "" }),
    /check exited 0/,
  );
  assert.throws(
    () => assertCliResult({ status: 1, stdout: JSON.stringify(clean), stderr: "" }),
    /did not report agents-md-max-size/,
  );
  finding.rules[0].line = 2;
  assert.throws(
    () => assertCliResult({ status: 1, stdout: JSON.stringify(finding), stderr: "" }),
    /did not report agents-md-max-size/,
  );
});

test("the compile step builds exactly the targets the test step runs, same profile", () => {
  for (const [os, { rust }] of systems) {
    const build = buildCommand(os);
    const [first] = cargoCommands(os);
    const prefix = first.slice(0, first.indexOf("no-mistakes") + 1);
    assert.deepEqual(build.slice(0, prefix.length), prefix);
    assert.equal(build[prefix.length], "--no-run");
    const selectors = build.slice(prefix.length + 1);
    assert.deepEqual(
      selectors,
      [...new Set(rust.map((entry) => entry.selector.join(" ")))].flatMap((s) => s.split(" ")),
    );
    for (const args of cargoCommands(os)) assert.deepEqual(args.slice(0, prefix.length), prefix);
  }
});

test("the macOS smoke job runs on pull requests and drives the checked-in list", () => {
  const body = workflowJob(read(".github/workflows/ci.yml"), "macos-smoke");
  assert.match(body, /needs\.changes\.outputs\.macos-smoke == 'true'/);
  assert.match(body, /runs-on: macos-15/);
  assert.doesNotMatch(body, /runs-on: windows/);
  for (const mode of ["rust", "cli", "napi", "api"])
    assert.match(body, new RegExp(`smoke\\.cjs macos ${mode}`));
  assert.doesNotMatch(body, /pull_request_target|secrets\./);
});

test("the Windows smoke job runs on pull requests and drives the checked-in list", () => {
  const body = workflowJob(read(".github/workflows/ci.yml"), "windows-smoke");
  assert.match(body, /needs\.changes\.outputs\.windows-smoke == 'true'/);
  assert.match(body, /runs-on: windows-2025/);
  assert.match(body, /timeout-minutes: 45/);
  assert.match(body, /permissions:\n {6}contents: read\n/);
  assert.match(body, /github\.event\.pull_request\.head\.repo\.full_name == github\.repository/);
  assert.match(body, /native-runtime-workspace-v2-x86_64-pc-windows-msvc-/);
  assert.doesNotMatch(body, /macos/i);
  for (const mode of ["rust", "cli", "napi", "api"])
    assert.match(body, new RegExp(`smoke\\.cjs windows ${mode}`));
  assert.doesNotMatch(body, /pull_request_target|secrets\./);
});

test("the macOS smoke Rust tests compile with the profile the main test-harness cache holds", () => {
  const { commands } = require("./native-rust-tests.cjs");
  const cached = commands("build", "macOS")[0];
  const [smoke] = cargoCommands("macos");
  const profile = (args) => args.slice(0, args.indexOf("--locked"));
  assert.deepEqual(profile(smoke), profile(cached));
  assert.deepEqual(smoke.slice(smoke.indexOf("-p"), smoke.indexOf("-p") + 2), [
    "-p",
    "no-mistakes",
  ]);
});
