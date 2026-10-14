const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { createHash } = require("node:crypto");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const test = require("node:test");

const script = path.join(__dirname, "benchmark-handoff.sh");
const fixtures = path.join(__dirname, "../../fixtures/benchmark-handoff");
const archive = path.join(fixtures, "valid/archive.txt");
const digest = createHash("sha256").update(readFileSync(archive)).digest("hex");
const run = (args, env = {}) =>
  spawnSync("bash", [script, ...args], {
    encoding: "utf8",
    env: { ...process.env, CACHE_HIT: "", ARTIFACT_ID: "", BENCHMARK_SHA256: "", ...env },
  });

test("only the producer's exact archive digest is accepted", () => {
  assert.equal(run(["verify", archive], { BENCHMARK_SHA256: digest }).status, 0);
  const result = run(["verify", path.join(fixtures, "tampered/archive.txt")], {
    BENCHMARK_SHA256: digest,
  });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /refusing to extract or execute/);
});

test("missing archives and malformed producer digests fail closed", () => {
  assert.equal(
    run(["verify", path.join(fixtures, "missing")], { BENCHMARK_SHA256: digest }).status,
    1,
  );
  for (const BENCHMARK_SHA256 of ["", "not-a-digest", `${digest}\nother`]) {
    const result = run(["verify", archive], { BENCHMARK_SHA256 });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /invalid benchmark digest/);
  }
});

test("a cache-save warning cannot hide failure of both transports", () => {
  for (const CACHE_HIT of ["", "false"]) {
    const result = run(["available"], { CACHE_HIT });
    assert.equal(result.status, 1);
    assert.match(result.stderr, /No exact benchmark cache or uploaded fallback/);
  }
});

test("an exact cache probe or an uploaded artifact independently suffices", () => {
  assert.equal(run(["available"], { CACHE_HIT: "true" }).status, 0);
  assert.equal(run(["available"], { CACHE_HIT: "false", ARTIFACT_ID: "123" }).status, 0);
  assert.equal(run(["available"], { CACHE_HIT: "true", ARTIFACT_ID: "123" }).status, 0);
  assert.equal(run(["available"], { ARTIFACT_ID: "invalid" }).status, 1);
});

test("unknown guard modes fail before accepting a transport", () => {
  assert.equal(run([]).status, 2);
  assert.equal(run(["other"]).status, 2);
});
