const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { run } = require("./postgres-source-identities.cjs");
const { libraryShardCoverage } = require("./standalone-shard-modules.cjs");
const { workflowJob } = require("./workflow-contract.cjs");
const fixture = require("./fixtures/postgres-source-identities.json");

test("the source filter compiles one bounded postgres-source test slice", () => {
  const calls = [];
  assert.equal(
    run({ RUSTFLAGS: "-D warnings" }, (command, args, options) => {
      calls.push({ command, args, ...options });
      return { status: 0 };
    }),
    0,
  );
  assert.deepEqual(
    calls.map((call) => call.env.NM_TEST_SHARD),
    fixture.shards,
  );
  for (const call of calls) {
    assert.equal(call.command, "cargo");
    assert.deepEqual(call.args, fixture.args);
    assert.equal(call.env.CARGO_BUILD_JOBS, "1");
    assert.equal(call.env.CARGO_PROFILE_TEST_CODEGEN_UNITS, "4096");
    assert.equal(call.env.CARGO_PROFILE_TEST_DEBUG, "0");
    assert.equal(call.env.CARGO_INCREMENTAL, "0");
    assert.equal(call.env.RUSTFLAGS, "-D warnings");
    assert.equal(call.env.RUSTC_WRAPPER, path.join(__dirname, "standalone-shard-rustc.sh"));
  }
});

test("every no-mistakes-postgres-sql source unit test compiles in the postgres-source slice", () => {
  const root = path.join(__dirname, "../..");
  const report = libraryShardCoverage(root);
  assert.deepEqual(report.missingPostgresSourceTests, []);
  assert.ok(report.postgresSourceTestCount >= 328, String(report.postgresSourceTestCount));
});
test("failed slices fail immediately instead of being retried or hidden", () => {
  let calls = 0;
  assert.equal(
    run({ CARGO_BUILD_JOBS: "8" }, (_command, _args, options) => {
      calls += 1;
      assert.equal(options.env.CARGO_BUILD_JOBS, "8");
      return { status: 17 };
    }),
    17,
  );
  assert.equal(calls, 1);
});
test("the workflow selects bounded compilation without raising the deadlines", () => {
  const workflow = readFileSync(path.join(__dirname, "../workflows/ci.yml"), "utf8");
  const job = workflowJob(workflow, "quality-rust");
  assert.match(
    job,
    /name: PostgreSQL source identities\n\s+command: .*with-resource-samples\.sh node \.github\/scripts\/postgres-source-identities\.cjs/,
  );
  assert.doesNotMatch(job, /command: .*cargo nextest run.*codebase::postgres::source::/);
  const deadlines = [
    ...job.matchAll(/matrix\.name == 'PostgreSQL source identities' && (\d+) \|\| (\d+)/g),
  ];
  assert.deepEqual(
    deadlines.map((match) => [Number(match[1]), Number(match[2])]),
    [
      [fixture.jobMinutes, 25],
      [fixture.stepMinutes, 18],
    ],
  );
  assert.match(workflow, /node --test .*postgres-source-identities\.test\.cjs/);
});

test("a previously selected shard wrapper is not forwarded to itself", () => {
  const wrapper = path.join(__dirname, "standalone-shard-rustc.sh");
  assert.equal(
    run({ RUSTC_WRAPPER: wrapper }, (_command, _args, options) => {
      assert.equal(options.env.NM_INNER_RUSTC_WRAPPER, "");
      return { status: 0 };
    }),
    0,
  );
});
