const assert = require("node:assert/strict");
const { readdirSync, readFileSync } = require("node:fs");
const { join } = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");
const { libraryShardCoverage } = require("./standalone-shard-modules.cjs");
const evidence = require("./fixtures/standalone-build-resources.json");

const root = join(__dirname, "../..");
const workflow = readFileSync(join(root, ".github/workflows/ci.yml"), "utf8");
const producer = workflowJob(workflow, "standalone-build");
const docs = readFileSync(join(root, "docs/development/ci-builds.md"), "utf8");

test("standalone library compilation stays inside the 16 GB runner", () => {
  assert.ok(evidence.before.peakRssMib > evidence.ceilingRssMib);
  assert.ok(evidence.after.peakRssMib < evidence.ceilingRssMib);
  assert.ok(evidence.after.peakRssMib < evidence.before.peakRssMib);
  assert.ok(evidence.noSwapFailure.peakRssMib > evidence.ceilingRssMib);
  assert.equal(evidence.noSwapFailure.oomKills, 1);
  const settings = producer.split("    steps:\n")[0];
  for (const [key, value] of Object.entries(evidence.settings))
    assert.match(settings, new RegExp(`${key}: "${value}"`));
  assert.doesNotMatch(settings, /CARGO_PROFILE_(?:DEV|TEST)_DEBUG: "(?!0")/);
  assert.match(
    producer,
    /printf 'standalone library build: cargo_jobs=%s dev_units=%s test_units=%s dev_debug=%s test_debug=%s incremental=%s shards=%s\\n'/,
  );
  assert.match(producer, /"\$CARGO_INCREMENTAL" 3/);
  const shards = workflowJob(workflow, "standalone-shards");
  assert.match(shards, /shard: \[1, 2, 3\]/);
  assert.match(shards, /Standalone library shard \(\$\{\{ matrix\.shard \}\}\/3\)/);
  assert.match(
    shards,
    /standalone-shards\.cjs run target\/standalone-bundle \$\{\{ matrix\.shard \}\} 3/,
  );
  const helper = readFileSync(join(root, ".github/scripts/standalone-shards.cjs"), "utf8");
  assert.match(helper, /const SHARDS = 3/);
  assert.match(helper, /NM_TEST_SHARD/);
  assert.match(helper, /standalone-shard-rustc\.sh/);
  assert.doesNotMatch(helper, /--partition/);
  const wrapper = readFileSync(join(root, ".github/scripts/standalone-shard-rustc.sh"), "utf8");
  assert.match(wrapper, /no_mistakes/);
  assert.match(wrapper, /"-A" "dead_code"/);
  assert.match(wrapper, /"-A" "unused_imports"/);
  for (const value of Object.values(evidence.localFourCpu)) {
    assert.ok(value > 0);
    assert.match(docs, new RegExp(String(value)));
  }
  assert.ok(evidence.localFourCpu.codegenUnits4096RssMib > evidence.ceilingRssMib);
  assert.ok(evidence.localFourCpu.codegenUnits16384RssMib > evidence.ceilingRssMib);
  assert.ok(evidence.localFourCpu.libraryWithoutTestModulesRssMib < evidence.ceilingRssMib);
  assert.ok(evidence.localFourCpu.shardPeakRssMib < evidence.ceilingRssMib);
  assert.match(
    producer,
    /bash \.github\/scripts\/with-resource-samples\.sh node \.github\/scripts\/standalone-shards\.cjs build target\/standalone-bundle/,
  );
  assert.ok(
    producer.indexOf("standalone library build:") < producer.indexOf("with-resource-samples.sh"),
  );
  assert.match(
    producer,
    /cgu\$\{\{ env\.CARGO_PROFILE_TEST_CODEGEN_UNITS \}\}-debug\$\{\{ env\.CARGO_PROFILE_TEST_DEBUG \}\}/,
  );
  assert.match(docs, new RegExp(evidence.before.runId));
  assert.match(docs, new RegExp(String(evidence.before.peakRssMib)));
  assert.match(docs, new RegExp(evidence.after.runId));
  assert.match(docs, new RegExp(String(evidence.after.peakRssMib)));
  assert.match(docs, new RegExp(evidence.noSwapFailure.runId));
  assert.match(docs, new RegExp(String(evidence.ceilingRssMib)));
  for (const value of Object.values(evidence.settings)) assert.match(docs, new RegExp(value));
});

function rustFiles(dir, out = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const full = join(dir, entry.name);
    if (entry.isDirectory()) rustFiles(full, out);
    else if (entry.name.endsWith(".rs")) out.push(full);
  }
  return out;
}

test("unit-test modules are split across three compile shards", () => {
  const counts = { 0: 0, 1: 0, 2: 0 };
  const shardLine =
    /^(\s*)#\[cfg\(all\(test, any\(nm_shard_all, nm_shard_(\d)(?:, nm_shard_postgres_source)?\)\)\)\]\s*$/;
  const exact = /^(\s*)#\[cfg\(test\)\]\s*$/;
  const attr = /^\s*#\[/;
  const modRe = /^(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z0-9_]+)\b/;
  const includeRe = /^include!\("([^"]+)"\)\s*;\s*$/;
  let helpers = 0;
  for (const file of rustFiles(join(root, "crates/no-mistakes/src"))) {
    const lines = readFileSync(file, "utf8").split("\n");
    for (let index = 0; index < lines.length; index += 1) {
      const sharded = lines[index].match(shardLine);
      if (sharded) {
        assert.ok(sharded[2] === "0" || sharded[2] === "1" || sharded[2] === "2", file);
        counts[sharded[2]] += 1;
        continue;
      }
      if (!exact.test(lines[index])) continue;
      let cursor = index + 1;
      while (cursor < lines.length && (lines[cursor].trim() === "" || attr.test(lines[cursor])))
        cursor += 1;
      const target = (lines[cursor] ?? "").trim();
      const mod = target.match(modRe);
      const include = target.match(includeRe);
      const governs =
        (mod && (mod[1] === "tests" || mod[1].endsWith("_tests"))) ||
        (include && include[1].endsWith("tests.rs"));
      if (governs) helpers += 1;
    }
  }
  const total = counts[0] + counts[1] + counts[2];
  assert.ok(total > 1200, String(total));
  for (const id of ["0", "1", "2"]) assert.ok(counts[id] >= total * 0.2, `${id}=${counts[id]}`);
  assert.ok(helpers > 0);
  assert.ok(helpers < 300, String(helpers));
  const buildRs = readFileSync(join(root, "crates/no-mistakes/build.rs"), "utf8");
  assert.match(buildRs, /nm_shard_all/);
  assert.match(buildRs, /NM_TEST_SHARD must be 0, 1, 2, or postgres-source/);
});

test("nested test modules inherit the parent shard", () => {
  const report = libraryShardCoverage(root);
  assert.deepEqual(report.unresolved, []);
  assert.deepEqual(
    report.nested.map((item) => `${item.file}:${item.line}:${item.name}`),
    [],
  );
  assert.deepEqual(
    report.droppedTests.map((item) => `${item.file}:${item.line}:${item.name}`),
    [],
  );
  assert.deepEqual(report.incomplete, []);
  assert.equal(report.testCount, report.plainTestCount);
  assert.ok(report.testCount > 10000, String(report.testCount));
});

test("a char literal before a sharded test module stays visible to the lock", () => {
  const fixture = join(root, "test-cases/standalone-shards/char-literal");
  const report = libraryShardCoverage(fixture, {
    entry: join(fixture, "src/lib.rs"),
    srcDir: join(fixture, "src"),
  });
  assert.deepEqual(report.incomplete, []);
  assert.deepEqual(report.unresolved, []);
  assert.deepEqual(
    report.nested.map((item) => `${item.name}:${item.shard}:${item.parentShards.join(",")}`),
    ["child:0:2"],
  );
  assert.deepEqual(
    report.droppedTests.map((item) => item.name),
    ["child_test"],
  );
  assert.equal(report.testCount, 1);
  assert.equal(report.plainTestCount, 1);
});
