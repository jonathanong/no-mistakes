const assert = require("node:assert/strict");
const { test } = require("node:test");
const { probe, darwinDiagnostics } = require("./native-build-probes.cjs");
const saved = require("../../test-cases/native-build-diagnostics/reports/darwin-probes.json");

test("Darwin diagnostics retain bounded memory, full process and thread probes", () => {
  const calls = [];
  const result = darwinDiagnostics((...args) => {
    calls.push(args);
    return args[0] === "memory_pressure" ? saved.failure : saved.success;
  });
  assert.deepEqual(calls, saved.commands);
  assert.deepEqual(result, {
    virtualMemory: saved.success,
    memoryPressure: saved.failure,
    processes: saved.success,
    threads: saved.success,
  });
});

test("native probes retain subprocess output, failures and timeouts", () => {
  const success = probe(process.execPath, ["--version"]);
  assert.equal(success.status, 0);
  assert.match(success.stdout, /^v\d+/);
  assert.equal(success.signal, null);
  const missing = probe("no-mistakes-missing-diagnostic-command", []);
  assert.equal(missing.status, null);
  assert.match(missing.error, /ENOENT/);
  const timed = probe(
    process.execPath,
    [require.resolve("../../test-cases/native-build-diagnostics/diagnostic-wait.cjs")],
    100,
  );
  assert.equal(timed.status, null);
  assert.match(timed.error, /ETIMEDOUT/);
});
