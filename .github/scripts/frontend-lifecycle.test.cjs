const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { test } = require("node:test");

const executable = process.env.NO_MISTAKES_BENCH_EXECUTABLE;
const native = process.env.NO_MISTAKES_BENCH_NATIVE_CONTROL === "1";
const options = { skip: !executable };

function run(overrides = {}, filter = "") {
  const env = {
    ...process.env,
    RAYON_NUM_THREADS: "4",
    NO_MISTAKES_BENCH_SHARD: "language-frontends",
    NO_MISTAKES_FRONTEND_LIFECYCLE: "cold_first_analysis",
    ...overrides,
  };
  for (const key of Object.keys(env)) {
    if (env[key] === undefined) delete env[key];
  }
  return spawnSync(executable, [...(native ? ["--test"] : []), ...(filter ? [filter] : [])], {
    env,
    encoding: "utf8",
    timeout: 30_000,
  });
}

test("fresh cold process runs one closure and retains full fixture preflight", options, () => {
  const result = run();
  assert.equal(result.status, 0, result.stdout + result.stderr);
  const lines = result.stdout.split("\n").filter((line) => line.startsWith("FRONTEND_LIFECYCLE "));
  assert.equal(lines.length, 1);
  const ledger = JSON.parse(lines[0].slice("FRONTEND_LIFECYCLE ".length));
  assert.deepEqual(ledger, {
    mode: "cold_first_analysis",
    pid: result.pid,
    priorCollectors: 0,
    measuredClosures: 1,
    measuredCollectors: 1,
    postMarkerCollectors: 1,
    rayonWorkers: 4,
    files: 117,
    parsed: 69,
    imports: 66,
    enqueues: 7,
    workers: 6,
    routes: 41,
    edges: 125,
    queueEdges: 14,
    globMatches: 117,
  });
});

test("selection rejects unknown modes, broad shards, and a different worker count", options, () => {
  // Selection is the first Criterion target: invalid modes must fail before
  // any other shard can perform its own preflight or pool initialization.
  for (const overrides of [
    { NO_MISTAKES_FRONTEND_LIFECYCLE: "unknown" },
    { NO_MISTAKES_BENCH_SHARD: undefined },
    { NO_MISTAKES_BENCH_SHARD: "native-frontends" },
    { RAYON_NUM_THREADS: "1" },
  ]) {
    const result = run(overrides);
    assert.notEqual(result.status, 0);
    assert.doesNotMatch(result.stdout, /Testing |Measured:|Checked:|FRONTEND_LIFECYCLE /);
  }
});

test(
  "a filter that skips the cold closure fails instead of reporting fabricated work",
  options,
  () => {
    const result = run({}, "no-such-benchmark");
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /cold benchmark was filtered out or repeated/);
  },
);

test("original registration remains available in the same executable", options, () => {
  const result = run({ NO_MISTAKES_FRONTEND_LIFECYCLE: undefined });
  assert.equal(result.status, 0, result.stdout + result.stderr);
  assert.match(result.stdout, /language_frontends(?:::|\/)extract/);
  assert.match(result.stdout, /language_frontends(?:::|\/)edges/);
  assert.match(result.stdout, /language_frontends(?:::|\/)queue_glob_match/);
  assert.doesNotMatch(result.stdout, /FRONTEND_LIFECYCLE /);
});

test(
  "native one-sample statistics are rejected before analysis",
  { skip: !executable || !native },
  () => {
    // Criterion 5.0.1 otherwise executes once and panics in sample statistics.
    // Functional --test controls must stay distinct from sampled native timing.
    const result = spawnSync(executable, ["--bench"], {
      env: {
        ...process.env,
        RAYON_NUM_THREADS: "4",
        NO_MISTAKES_BENCH_SHARD: "language-frontends",
        NO_MISTAKES_FRONTEND_LIFECYCLE: "cold_first_analysis",
      },
      encoding: "utf8",
      timeout: 30_000,
    });
    assert.notEqual(result.status, 0);
    assert.match(result.stderr, /native lifecycle controls require --test/);
    assert.doesNotMatch(result.stdout, /FRONTEND_LIFECYCLE /);
  },
);
