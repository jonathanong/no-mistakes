const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { cpSync, mkdtempSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const { readWorkflow } = require("./workflow-contract.cjs");
const toolPaths = require("./fixtures/release-parity/tools-paths.json");

const root = path.resolve(__dirname, "../..");
const historical = path.join(root, "test-cases/rust-validation/fixture/historical-release");

test("release Rust parity runs from tagged source with isolated dependency-free tools", () => {
  const workflow = readWorkflow(path.join(root, ".github/workflows/release.yml"));
  const parity = workflow.jobs["validate-rust"].steps.find(
    (step) => step.name === "Check Rust validation parity",
  );
  assert.ok(parity?.run, "release must invoke the Rust validation parity suite");

  const sandbox = mkdtempSync(path.join(tmpdir(), "release-parity-"));
  try {
    const source = path.join(sandbox, "source");
    const runnerTemp = path.join(sandbox, "runner-temp");
    const tools = path.join(runnerTemp, "validation-tools");
    cpSync(historical, source, { recursive: true });
    for (const relative of toolPaths) {
      cpSync(path.join(root, relative), path.join(tools, relative), {
        recursive: true,
        filter: (file) => ![".git", "node_modules"].includes(path.basename(file)),
      });
    }

    const preload = path.join(
      tools,
      ".github/scripts/fixtures/workflow-contract/builtins-only.cjs",
    );
    const env = { ...process.env };
    delete env.NODE_TEST_CONTEXT;
    const result = spawnSync(
      "bash",
      ["--noprofile", "--norc", "-eo", "pipefail", "-c", parity.run],
      {
        cwd: source,
        encoding: "utf8",
        env: {
          ...env,
          NODE_OPTIONS: `--require ${JSON.stringify(preload)}`,
          NODE_PATH: "",
          RUNNER_TEMP: runnerTemp,
        },
        timeout: parity["timeout-minutes"] * 60_000,
      },
    );
    const output = result.stdout + result.stderr;
    assert.ifError(result.error);
    assert.equal(result.status, 0, output);
    assert.match(output, /release retains the shared all-feature test and doctest scopes/);
  } finally {
    rmSync(sandbox, { recursive: true, force: true });
  }
});
