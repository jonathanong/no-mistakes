const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const path = require("node:path");
const { test } = require("node:test");
const { readWorkflow } = require("./workflow-contract.cjs");

const workflow = readWorkflow(path.join(__dirname, "../workflows/ci.yml"));
const linuxApi = workflow.jobs["linux-api-tests"];

test("Linux real N-API tests cover four isolated matrix shards", () => {
  assert.equal(linuxApi.name, "Linux N-API tests (${{ matrix.shard }}/4)");
  assert.equal(linuxApi.strategy["fail-fast"], false);
  assert.deepEqual(linuxApi.strategy.matrix.shard, [1, 2, 3, 4]);
  assert.equal(linuxApi.env.NATIVE_API_SHARD, "${{ matrix.shard }}");

  const run = linuxApi.steps.find((step) => step.name === "Run real N-API API test");
  assert.ok(run);
  assert.ok(run["timeout-minutes"] <= 4);
  assert.match(run.run, /native-api-shards\.cjs\s+"\$NATIVE_API_SHARD"\s+4\s+linux/);
});

test("each Linux N-API shard enforces the slow-test gate and uploads a unique report", () => {
  const reporter = linuxApi.steps.find((step) => step.name === "Report real N-API test durations");
  assert.ok(reporter);
  assert.equal(typeof reporter.run, "string");
  assert.ok(reporter.run.length > 0);
  assert.match(reporter.run, /report-native-api-tests\.cjs/);
  assert.doesNotMatch(reporter.run, /--report-only/);
  assert.match(reporter.if, /steps\.real-napi\.outcome != 'skipped'/);

  const upload = linuxApi.steps.find((step) => step.name === "Upload real N-API test durations");
  assert.ok(upload);
  assert.match(upload.if, /hashFiles\('target\/native-api-tests\.json'\) != ''/);
  assert.equal(upload.with.name, "native-api-tests-linux-${{ matrix.shard }}");
});

test("JavaScript aggregate fails when the Linux N-API matrix fails or skips", () => {
  const aggregate = workflow.jobs["javascript-tests"];
  assert.ok(aggregate.needs.includes("linux-api-tests"));
  assert.equal(aggregate.if, "${{ !cancelled() }}");
  const script = aggregate.steps.find((step) => step.name === "Require all component jobs").run;
  for (const result of ["success", "failure", "cancelled", "skipped"]) {
    const child = spawnSync("bash", ["-eu", "-c", script], {
      encoding: "utf8",
      env: {
        ...process.env,
        RESULTS: JSON.stringify({
          "javascript-coverage": { result: "success" },
          "linux-api-tests": { result },
        }),
      },
    });
    assert.equal(child.error, undefined, `${result}: aggregate command should start`);
    if (result === "success") {
      assert.equal(child.status, 0, "all-success dependencies must pass aggregate");
    } else {
      assert.notEqual(child.status, 0, `${result} Linux matrix result must fail aggregate`);
    }
  }
});

test("Linux shards use one bounded verified download for both runtime components", () => {
  const download = linuxApi.steps.find((step) => step.name === "Download Linux runtime");
  assert.equal(download["timeout-minutes"], 6);
  assert.equal(
    download.env.NATIVE_RUNTIME_RUN_ID,
    "${{ needs.linux-runtime.outputs.artifact-run-id }}",
  );
  assert.equal(download.env.GH_TOKEN, "${{ github.token }}");
  assert.equal(
    download.run,
    "python3 .github/scripts/download-native-runtime.py linux-runtime-cli linux-runtime-addon",
  );
  assert.equal(download.uses, undefined);
});

test("shared producer artifact contracts run before dependency installation", () => {
  const result = spawnSync(
    process.execPath,
    [
      "--require",
      path.join(__dirname, "fixtures/workflow-contract/builtins-only.cjs"),
      "--test",
      path.join(__dirname, "build-artifact-workflow.test.cjs"),
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.status, 0, result.stdout + result.stderr);
});

test("AST analysis downloads only the verified CLI through the bounded helper", () => {
  const ast = workflow.jobs["ast-analysis"];
  assert.equal(ast.needs, "linux-runtime");
  const downloads = ast.steps.filter((step) => step.name === "Download Linux runtime");
  assert.equal(downloads.length, 1);
  const download = downloads[0];
  assert.equal(download["timeout-minutes"], 6);
  assert.equal(
    download.env.NATIVE_RUNTIME_RUN_ID,
    "${{ needs.linux-runtime.outputs.artifact-run-id }}",
  );
  assert.equal(download.env.GH_TOKEN, "${{ github.token }}");
  assert.equal(
    download.run,
    "python3 .github/scripts/download-native-runtime.py linux-runtime-cli",
  );
  assert.equal(download.uses, undefined);
  assert.equal(ast.steps.filter((step) => step.uses?.startsWith("pnpm/setup@")).length, 0);
});

test("native runtime download jobs outlive the download step and earlier step bounds", () => {
  const covered = [];
  for (const [jobId, job] of Object.entries(workflow.jobs)) {
    const steps = Array.isArray(job.steps) ? job.steps : [];
    const downloadAt = steps.findIndex(
      (step) => typeof step.run === "string" && step.run.includes("download-native-runtime.py"),
    );
    if (downloadAt < 0) continue;
    covered.push(jobId);
    const required = steps.slice(0, downloadAt + 1).reduce((sum, step, index) => {
      const minutes = step["timeout-minutes"];
      assert.equal(typeof minutes, "number", `${jobId} step ${index} needs a literal timeout`);
      return sum + minutes;
    }, 0);
    const jobMinutes = job["timeout-minutes"];
    assert.equal(typeof jobMinutes, "number", `${jobId} needs a literal job timeout`);
    assert.ok(
      jobMinutes >= required,
      `${jobId} timeout-minutes ${jobMinutes} is below ${required}, the sum of step bounds through the download`,
    );
  }
  assert.deepEqual(covered.sort(), [
    "ast-analysis",
    "linux-api-tests",
    "native-api-tests",
    "native-api-tests-macos",
  ]);
});
