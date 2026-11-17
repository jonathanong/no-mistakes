const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { parse } = require("yaml");

const root = path.resolve(__dirname, "../..");
const workflow = parse(readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8"));

test("Linux N-API cases run in three bounded shards with the existing failure gate", () => {
  const quality = workflow.jobs["quality-source"];
  const setupIndex = quality.steps.findIndex((entry) => entry.name === "Install pnpm dependencies");
  const parityIndex = quality.steps.findIndex((entry) => entry.name === "Check Rust validation parity");
  assert.ok(setupIndex >= 0 && parityIndex > setupIndex);
  assert.match(quality.steps[parityIndex].run, /linux-native-api-workflow\.test\.cjs/);
  assert.doesNotMatch(
    readFileSync(path.join(__dirname, "build-native.test.cjs"), "utf8"),
    /linux-native-api-workflow\.test\.cjs/,
  );

  const job = workflow.jobs["linux-api-tests"];
  assert.ok(job);
  assert.equal(job.needs, "linux-runtime");
  assert.equal(job["runs-on"], "ubuntu-latest");
  assert.equal(job["timeout-minutes"], 8);
  assert.equal(job.strategy["fail-fast"], false);
  assert.deepEqual(job.strategy.matrix.shard, [1, 2, 3]);

  const step = (name) => job.steps.find((entry) => entry.name === name);
  const download = step("Download Linux runtime");
  assert.match(download.uses, /^actions\/download-artifact@[0-9a-f]{40}$/);
  assert.equal(download.with["pattern"], "linux-runtime-*");
  assert.equal(download.with["merge-multiple"], true);

  const run = step("Run real N-API API test");
  assert.equal(run["timeout-minutes"], 4);
  assert.equal(run.id, "real-napi");
  assert.equal(run.env.NATIVE_API_SHARD, "${{ matrix.shard }}");
  assert.equal(
    run.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH,
    "${{ github.workspace }}/target/debug/no-mistakes.node",
  );
  assert.match(run.run, /native-api-shards\.cjs "\$NATIVE_API_SHARD" 3 linux/);

  const report = step("Report real N-API test durations");
  assert.equal(report["timeout-minutes"], 2);
  assert.equal(report.if, "${{ !cancelled() && steps.real-napi.outcome != 'skipped' }}");
  assert.match(report.run, /report-native-api-tests\.cjs target\/native-api-tests\.json target/);
  assert.doesNotMatch(report.run, /--report-only/);

  const upload = step("Upload real N-API test durations");
  assert.equal(upload["continue-on-error"], true);
  assert.equal(upload.with.name, "native-api-tests-linux-${{ matrix.shard }}");
  assert.match(upload.with.path, /target\/native-api-inventory\.json/);
  assert.match(upload.with.path, /target\/native-api-shard\.json/);
  assert.match(upload.with.path, /target\/native-api-tests\.json/);
  assert.match(upload.with.path, /target\/native-api-tests-durations\.csv/);
  assert.match(upload.with.path, /target\/native-api-tests-summary\.md/);

  const aggregate = workflow.jobs["javascript-tests"];
  assert.deepEqual(aggregate.needs, ["javascript-coverage", "linux-api-tests"]);
  assert.equal(aggregate.if, "${{ always() }}");
  assert.equal(aggregate.steps[0].env.RESULTS, "${{ toJSON(needs) }}");
  assert.match(aggregate.steps[0].run, /job\.result!=="success"/);
});
