const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const path = require("node:path");
const { test } = require("node:test");
const { readWorkflow } = require("./workflow-contract.cjs");

test("AST analysis requires an empty fixture baseline without a distinct base", () => {
  const workflow = readWorkflow(path.join(__dirname, "../workflows/ci.yml"));
  const step = workflow.jobs["ast-analysis"].steps.find(({ name }) => name === "Run ast-grep scan");
  const head = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
  const base = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
  const commands = [
    'node() { printf "%s\\n" "$TEST_BASE"; }',
    'git() { if [[ "$1" != cat-file ]]; then printf "%s\\n" "$*"; fi; }',
    "ast-grep() { :; }",
    'python3() { printf "%s\\n" "$*"; }',
    step.run,
  ].join("\n");

  const baselineCheck = ".ast-grep/check-inline-fixture-writes.py";
  for (const [resolvedBase, expected] of [
    ["", `${baselineCheck} --require-empty-baseline`],
    ["0".repeat(40), `${baselineCheck} --require-empty-baseline`],
    [head, `${baselineCheck} --require-empty-baseline`],
    [base, `${baselineCheck} --base ${base}`],
  ]) {
    const child = spawnSync("bash", ["-eu", "-c", commands], {
      encoding: "utf8",
      env: {
        ...process.env,
        GITHUB_EVENT_NAME: "push",
        GITHUB_REPOSITORY: "sample/widgets",
        GITHUB_REF_NAME: "main",
        GITHUB_RUN_ID: "400",
        HEAD_SHA: head,
        PR_BASE_SHA: "",
        TEST_BASE: resolvedBase,
      },
    });
    assert.equal(child.status, 0, child.stderr);
    assert.equal(child.stdout.trim(), expected, `resolved base: ${resolvedBase || "(empty)"}`);
  }
});
