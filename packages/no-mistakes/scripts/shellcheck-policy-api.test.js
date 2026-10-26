const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { dirname, join } = require("node:path");
const test = globalThis.test || require("node:test").test;
const addon = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH;

test(
  "compiled ShellCheck checks preserve unavailable-tool policy across public APIs",
  { skip: !addon?.endsWith(".node") },
  () => {
    const root = join(__dirname, "../../../test-cases/rules/shellcheck-runner/fixture/unavailable");
    const cli =
      process.env.NO_MISTAKES_TEST_CLI_PATH ||
      join(dirname(addon), process.platform === "win32" ? "no-mistakes.exe" : "no-mistakes");
    // A child isolates native process environment access from other test workers.
    const env = Object.fromEntries(
      Object.entries(process.env).filter(([key]) => key.toLowerCase() !== "path"),
    );
    env.PATH = "";
    const child = spawnSync(process.execPath, [join(root, "native-check.cjs"), cli], {
      env,
      encoding: "utf8",
    });
    assert.equal(child.error, undefined);
    assert.equal(child.status, 0, child.stderr);
    assert.deepEqual(JSON.parse(child.stdout), ["report", "advise", "ignore"]);
  },
);
