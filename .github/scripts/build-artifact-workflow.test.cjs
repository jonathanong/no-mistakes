const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");
const source = readFileSync(path.join(__dirname, "../workflows/ci.yml"), "utf8");
const producers = [
  "linux-runtime",
  "native-test-build",
  "standalone-build",
  "rust-coverage-build",
  "rust-coverage-build-integration",
];
const consumers = {
  "linux-api-tests": "linux-runtime",
  "ast-analysis": "linux-runtime",
  "native-rust-tests": "native-test-build",
  "standalone-shards": "standalone-build",
  "rust-coverage-shards": "rust-coverage-build",
  "rust-coverage-shards-integration": "rust-coverage-build-integration",
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
  const benchmarks = readFileSync(path.join(__dirname, "../workflows/benchmarks.yml"), "utf8");
  assert.match(benchmarks, /\|\| github.ref \}\}\n  cancel-in-progress:/);
  assert.match(benchmarks, /cancel-in-progress: \$\{\{ github.event_name == 'pull_request' \}\}/);
  assert.doesNotMatch(
    benchmarks.split("\njobs:")[0],
    /workflow_dispatch|ci\/cold-build|warm-build/,
  );
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
    if (
      name === "linux-api-tests" ||
      name === "ast-analysis" ||
      name.startsWith("native-api-tests")
    ) {
      assert.ok(
        body.includes(`NATIVE_RUNTIME_RUN_ID: \${{ needs.${producer}.outputs.artifact-run-id }}`),
      );
      assert.match(body, /GH_TOKEN: \$\{\{ github.token \}\}/);
      if (name.startsWith("native-api-tests"))
        assert.match(body, /NATIVE_RUNTIME_TARGET: \$\{\{ matrix.target \}\}/);
      assert.match(body, /python3? .github\/scripts\/download-native-runtime.py/);
      continue;
    }
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

test("warm validation installs LLVM tools before retaining the profile pooling test", () => {
  const body = workflowJob(source, "rust-coverage-build");
  const install = steps(body).find((step) => step.startsWith("Install Rust\n"));
  assert.ok(install);
  assert.doesNotMatch(install, /^\s*if:/m);
  assert.match(install, /components: llvm-tools-preview/);
  assert.ok(body.indexOf("name: Install Rust") < body.indexOf("rust-profile-pooling.test.cjs"));
  assert.match(body, /node --test[^\n]*rust-profile-pooling.test.cjs/);
});

test("full native API and macOS Rust tests run on their builders and download only validated reused artifacts", () => {
  for (const [name, build, upload, download, run] of [
    [
      "native-api-tests",
      "Build native CLI and N-API addon",
      "Upload native-runtime",
      "Download native-runtime",
      "Run real N-API API test",
    ],
    [
      "native-api-tests-macos",
      "Build native CLI and N-API addon",
      "Upload native-runtime",
      "Download native-runtime",
      "Run real N-API API test",
    ],
    [
      "native-rust-tests-macos",
      "Precompile native Rust test targets",
      "Upload native-rust-archive",
      "Download native-rust-archive",
      "Run platform-specific Rust tests",
    ],
  ]) {
    const body = workflowJob(source, name);
    const phases = steps(body);
    const index = (title) => phases.findIndex((step) => step.startsWith(`${title}\n`));
    for (const title of ["Validate build artifact source", build, upload, download, run])
      assert.ok(index(title) >= 0, `${name}: ${title}`);
    assert.match(body, /^    needs: changes$/m);
    assert.ok(
      body.includes(
        name === "native-rust-tests-macos" ? "timeout-minutes: 20" : "timeout-minutes: 65",
      ),
      name,
    );
    assert.ok(index("Validate build artifact source") < index(build), name);
    assert.ok(index(build) < index(upload) && index(upload) < index(run), name);
    assert.ok(index(download) < index(run), name);
    assert.match(phases[index(build)], /steps\.build-artifact-source.outputs.reuse != 'true'/);
    assert.match(phases[index(upload)], /steps\.build-artifact-source.outputs.reuse != 'true'/);
    assert.match(phases[index(download)], /steps\.build-artifact-source.outputs.reuse == 'true'/);
    assert.match(phases[index(download)], /steps\.build-artifact-source.outputs.run_id/);
    assert.doesNotMatch(phases[index(run)], /if:|outputs\.reuse|continue-on-error/);
    for (const phase of phases.filter((step) =>
      /uses: actions\/cache\/restore@|uses: Swatinem\/rust-cache@/.test(step),
    )) {
      assert.match(phase, /env.COLD_BUILD != 'true'/);
      assert.match(phase, /outputs.reuse != 'true'/);
    }
  }
});
