const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");
const source = readFileSync(path.join(__dirname, "../workflows/ci.yml"), "utf8");
const producers = [
  "linux-runtime",
  "native-build",
  "native-build-macos",
  "native-test-build",
  "native-test-build-macos",
  "standalone-build",
  "rust-coverage-build",
];
const consumers = {
  "linux-api-tests": "linux-runtime",
  "ast-analysis": "linux-runtime",
  "native-rust-tests": "native-test-build",
  "native-rust-tests-macos": "native-test-build-macos",
  "native-api-tests": "native-build",
  "native-api-tests-macos": "native-build-macos",
  "standalone-shards": "standalone-build",
  "rust-coverage-shards": "rust-coverage-build",
  tests: "rust-coverage-build",
};
const steps = (body) => body.split("      - name: ").slice(1);

test("cold and same-commit artifact measurements are explicit dispatch inputs", () => {
  assert.match(source, /workflow_dispatch:\n    inputs:\n      cold_cache:/);
  assert.match(source, /reuse_build_run:/);
  assert.match(source, /- ci\/cold-build/);
  assert.match(source, /- "ci\/warm-build-\*"/);
  assert.match(source, /github.ref == 'refs\/heads\/ci\/cold-build'/);
  assert.match(source, /COLD_BUILD: \$\{\{ \(inputs.cold_cache/);
  assert.match(source, /REUSE_BUILD_RUN_ID: \$\{\{ inputs.reuse_build_run/);
  assert.match(workflowJob(source, "changes"), /github.event_name != 'workflow_dispatch'/);
});

test("every build producer validates the source and publishes the artifact run ID", () => {
  for (const name of producers) {
    const body = workflowJob(source, name);
    assert.match(body, /artifact-run-id: \$\{\{ steps.build-artifact-source.outputs.run_id \}\}/);
    assert.match(body, /id: build-artifact-source/);
    assert.match(body, /node .github\/scripts\/build-artifact-source.cjs/);
    assert.ok(body.indexOf("name: Checkout") < body.indexOf("id: build-artifact-source"));
    for (const step of steps(body)) {
      if (step.startsWith("Test ")) assert.doesNotMatch(step.split("run:")[0], /outputs.reuse/);
      if (/uses: actions\/cache\/restore@|uses: Swatinem\/rust-cache@/.test(step)) {
        assert.match(step, /env.COLD_BUILD != 'true'/);
        assert.match(step, /outputs.reuse != 'true'/);
      }
      if (/uses: actions\/upload-artifact@/.test(step))
        assert.match(step, /outputs.reuse != 'true'/);
      if (
        /run:.*(?:cargo build|build-native.cjs|native-test-archive.cjs build)/s.test(step) &&
        !step.startsWith("Test ")
      )
        assert.match(step, /outputs.reuse != 'true'/);
    }
  }
});

test("test consumers download validated build sources while coverage profiles stay current", () => {
  for (const [name, producer] of Object.entries(consumers)) {
    const body = workflowJob(source, name);
    const downloads = steps(body).filter((step) => /uses: actions\/download-artifact@/.test(step));
    assert.ok(downloads.length > 0, name);
    for (const step of downloads) {
      if (name === "tests" && !step.includes("name: rust-coverage-build\n")) {
        assert.doesNotMatch(step, /run-id:/);
      } else {
        assert.ok(step.includes(`run-id: \${{ needs.${producer}.outputs.artifact-run-id }}`));
        assert.match(step, /github-token: \$\{\{ github.token \}\}/);
      }
    }
    assert.doesNotMatch(body, /if:.*inputs.reuse_build_run/);
  }
  assert.match(workflowJob(source, "tests-gate"), /name: Tests with coverage/);
  assert.match(workflowJob(source, "ast-analysis"), /name: AST analysis/);
});
