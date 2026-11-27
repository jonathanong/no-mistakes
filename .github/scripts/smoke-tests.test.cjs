const assert = require("node:assert/strict");
const { existsSync, readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");
const { assertCargoTestsRan, assertCliResult, cliCargoCommand } = require("./smoke.cjs");
const { cliCases, cliFixture, cliInvocation } = require("./smoke-tests.cjs");

const root = path.resolve(__dirname, "../..");
const read = (file) => readFileSync(path.join(root, file), "utf8");

test("each native CLI Cargo target and selected test case exists", () => {
  const manifest = read("crates/no-mistakes-invocation/Cargo.toml");
  assert.match(manifest, /\[\[test\]\]\nname = "cli_smoke"/);
  assert.ok(existsSync(path.join(root, cliFixture, ".no-mistakes.yml")));
  assert.throws(() => cliCargoCommand("linux"), /unsupported smoke OS/);
  for (const [os, names] of Object.entries(cliCases)) {
    const args = cliCargoCommand(os);
    assert.deepEqual(args.slice(0, 8), [
      "test", "--locked", "-p", "no-mistakes-invocation", "--test", "cli_smoke", "--", "--exact",
    ]);
    assert.deepEqual(args.slice(8), names.map((name) => `cli_invocation::${name}`));
    for (const name of names) assert.ok(read(cliInvocation).includes(`fn ${name}`), name);
  }
});

test("a Cargo filter that runs zero tests fails the smoke run", () => {
  const invocation = cliCargoCommand("macos");
  for (const output of ["running 0 tests\n\ntest result: ok. 0 passed\n", ""]) {
    assert.throws(() => assertCargoTestsRan(invocation, output), /ran zero tests/);
  }
  assert.doesNotThrow(() =>
    assertCargoTestsRan(invocation, "running 0 tests\r\nrunning 3 tests\r\n"),
  );
  assert.doesNotThrow(() => assertCargoTestsRan(invocation, "running 1 test\n"));
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

test("native API jobs run the same CLI cases with the platform runtime on every event", () => {
  const source = read(".github/workflows/ci.yml");
  for (const [os, id, binary, flag] of [
    ["macos", "native-api-tests-macos", "no-mistakes", "macos"],
    ["windows", "native-api-tests", "no-mistakes.exe", "windows"],
  ]) {
    const body = workflowJob(source, id);
    assert.ok(body.includes(`needs.changes.outputs.${flag} == 'true'`));
    const step = body
      .split("      - name: Run platform CLI invocation tests\n")[1]
      .split("\n      - ")[0];
    assert.match(step, /NO_MISTAKES_CLI_BIN:/);
    assert.ok(step.includes(`/target/debug/${binary}\n`));
    assert.ok(step.includes(`smoke.cjs ${os} cli-rust`));
    assert.match(step, /if: .* !cancelled\(\) && steps\.native-cli\.outcome == 'success'/);
    assert.doesNotMatch(step, /continue-on-error/);
    assert.match(body, /name: Run native CLI smoke test\n +id: native-cli/);
    assert.ok(
      body.indexOf("Upload real N-API test durations") <
        body.indexOf("Run platform CLI invocation tests"),
    );
    const toolchain = body
      .split("      - name: Install Rust for CLI invocation tests\n")[1]
      .split("\n      - ")[0];
    assert.ok(
      body.indexOf("Upload real N-API test durations") <
        body.indexOf("Install Rust for CLI invocation tests"),
    );
    assert.match(toolchain, /!cancelled\(\) && steps\.native-cli\.outcome == 'success'/);
    assert.match(step, /steps\.cli-rust-toolchain\.outcome == 'success'/);
    if (os === "macos") {
      assert.match(
        body,
        /id: native-rust-toolchain\n +if: .*steps\.build-artifact-source\.outputs\.reuse != 'true'/,
      );
      assert.match(toolchain, /steps\.build-artifact-source\.outputs\.reuse == 'true'/);
      assert.match(step, /steps\.build-artifact-source\.outputs\.reuse != 'true' \|\|/);
    }
    assert.ok(body.includes(`smoke.cjs ${os} cli ./target/debug/${binary}`));
    assert.ok(
      body.indexOf("Upload real N-API test durations") <
        body.indexOf("Run CLI check on a saved fixture"),
    );
  }
});
