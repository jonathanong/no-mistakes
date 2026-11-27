const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");
const evidence = require("./fixtures/coverage-build-resources.json");
const root = join(__dirname, "../..");
const workflow = readFileSync(join(root, ".github/workflows/ci.yml"), "utf8");
const producer = workflowJob(workflow, "rust-coverage-build");

test("instrumented compilation bounds PR Cargo jobs and serializes the largest units", () => {
  assert.ok(evidence.cargoBuildJobs < evidence.runnerCpus);
  const settings = producer.split("    steps:\n")[0];
  assert.ok(
    settings.includes(
      `CARGO_BUILD_JOBS: ${"${{"} github.event_name == 'pull_request' && '2' || '${evidence.cargoBuildJobs}' }}`,
    ),
  );
  assert.match(producer, /RUSTC_WRAPPER: .*pull_request.*coverage-rustc\.sh/);
  const wrapper = readFileSync(join(__dirname, "coverage-rustc.sh"), "utf8");
  assert.match(wrapper, /--crate-name.*no_mistakes/);
  assert.match(wrapper, /exec flock /);
  assert.doesNotMatch(producer, /CARGO_PROFILE_|link-arg|--no-default-features/);
  const archive = readFileSync(join(__dirname, "rust-coverage-shards.cjs"), "utf8");
  for (const flag of ["--locked", "--workspace", "--all-features", "--lib", "--bins", "--tests"])
    assert.ok(archive.includes(`"${flag}"`), flag);
  assert.match(producer, /source <\(cargo llvm-cov show-env --sh\)/);
});

test("coverage builds retain memory-pressure and exact cache restoration diagnostics", () => {
  assert.match(
    producer,
    /CACHE_MATCHED_KEY: \$\{\{ steps.coverage-workspace-cache.outputs.cache-matched-key \}\}/,
  );
  assert.match(
    producer,
    /CACHE_PRIMARY_KEY: \$\{\{ steps.coverage-workspace-cache.outputs.cache-primary-key \}\}/,
  );
  assert.match(producer, /printf 'coverage cache:/);
  assert.match(producer, /free -m/);
  assert.match(producer, /rustc -vV/);
  assert.match(producer, /Instrumented RUSTFLAGS=/);
  assert.match(
    producer,
    /bash \.github\/scripts\/with-resource-samples.sh node \.github\/scripts\/rust-coverage-shards.cjs archive target\/coverage-bundle/,
  );
  assert.ok(
    producer.indexOf("cargo llvm-cov show-env") < producer.indexOf("with-resource-samples.sh"),
  );
  const sampler = readFileSync(join(__dirname, "with-resource-samples.sh"), "utf8");
  for (const metric of [
    "mem_available_mib",
    "swap_free_mib",
    "oom_kills",
    "memory_full_avg10",
    "top_rss",
  ])
    assert.ok(sampler.includes(metric), metric);
});

test("only the proven intermittent historical Actions page is exempt from Lychee", () => {
  const patterns = readFileSync(join(root, ".lycheeignore"), "utf8")
    .split(/\r?\n/)
    .filter((line) => line && !line.startsWith("#"))
    .map((line) => new RegExp(line));
  assert.equal(patterns.filter((pattern) => pattern.test(evidence.historicalActionsUrl)).length, 1);
  for (const url of evidence.mustRemainChecked)
    assert.equal(
      patterns.some((pattern) => pattern.test(url)),
      false,
      url,
    );
});
