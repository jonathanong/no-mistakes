const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const os = require("node:os");
const { spawnSync } = require("node:child_process");
const { test } = require("node:test");
const saved = path.resolve(__dirname, "../../test-cases/libtest-argv-runner");
const cases = require("../../test-cases/libtest-argv-runner/cases.json");
const wrapper = path.join(__dirname, "libtest-argv-runner.sh");

test("missing binary fails before invoking a subprocess", () => {
  const result = spawnSync("bash", [wrapper], { encoding: "utf8" });
  assert.equal(result.status, 1);
  assert.equal(result.stdout, "");
  assert.match(result.stderr, /expected a nextest test binary/);
});

test(
  "target runner preserves real child argv, environment, output and exit status",
  { skip: process.platform === "win32" },
  () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "libtest-argv-runner-"));
    fs.cpSync(saved, directory, { recursive: true });
    const binary = path.join(directory, "harness.cjs");
    fs.chmodSync(binary, 0o755);
    const invoke = (args, env = {}) =>
      spawnSync("bash", [wrapper, binary, ...args], {
        encoding: "utf8",
        env: { ...process.env, NEXTEST_RUNNER_FIXTURE_VALUE: "retained", ...env },
      });
    try {
      for (const entry of cases) {
        const result = invoke(entry.args);
        assert.equal(result.status, 0, result.stderr);
        assert.deepEqual(JSON.parse(result.stdout), {
          args: entry.expected,
          inherited: "retained",
        });
        assert.equal(result.stderr, "fixture stderr preserved\n");
      }
      for (const status of [2, 17, 137]) {
        const result = invoke(cases[0].args, { NEXTEST_RUNNER_FIXTURE_STATUS: String(status) });
        assert.equal(result.status, status);
        assert.equal(result.signal, null);
      }
      for (const signal of ["SIGTERM", "SIGKILL"]) {
        const result = invoke(cases[0].args, { NEXTEST_RUNNER_FIXTURE_SIGNAL: signal });
        assert.equal(result.status, null);
        assert.equal(result.signal, signal);
      }
      const missing = spawnSync("bash", [wrapper, path.join(directory, "missing-binary")], {
        encoding: "utf8",
      });
      assert.equal(missing.status, 127);
      assert.equal(missing.stdout, "");
      assert.match(missing.stderr, /No such file or directory/);
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
