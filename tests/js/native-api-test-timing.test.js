const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const test = globalThis.test || require("node:test").test;
const { summarize, csv, markdown } = require("../../.github/scripts/report-native-api-tests.cjs");

const fixtureDir = path.join(__dirname, "../../fixtures/ci/native-api-tests");
const helper = path.join(__dirname, "../../.github/scripts/report-native-api-tests.cjs");
const fixture = (name) =>
  JSON.parse(fs.readFileSync(path.join(fixtureDir, `${name}.json`), "utf8"));

test("reports every completed and skipped test while keeping the 5s boundary exclusive", () => {
  const result = summarize(fixture("valid"));
  assert.equal(result.completed.length, 2);
  assert.equal(result.slowPassed.length, 0);
  assert.deepEqual(
    result.ranked.map((row) => row.durationMs),
    [4999.9, 1200.5],
  );
  assert.match(csv(result.rows), /"optional native case","pending",""/);
  // Node 20 is supported and has no Object.groupBy. Keep artifact rendering portable.
  const groupBy = Object.groupBy;
  try {
    Object.groupBy = undefined;
    assert.match(
      markdown(result),
      /Completed: 2; passed: 2; failed: 0; skipped: 0; pending: 1; todo: 0/,
    );
  } finally {
    Object.groupBy = groupBy;
  }
});

test("the CLI writes complete timing artifacts and rejects a passed test at 5s", () => {
  const outputDir = fs.mkdtempSync(path.join(os.tmpdir(), "native-api-timing-"));
  try {
    const completed = spawnSync(
      process.execPath,
      [helper, path.join(fixtureDir, "slow.json"), outputDir],
      { encoding: "utf8" },
    );
    assert.equal(completed.status, 1);
    assert.match(completed.stderr, /1 passed real N-API test\(s\) reached 5000 ms/);
    assert.match(
      fs.readFileSync(path.join(outputDir, "native-api-tests-durations.csv"), "utf8"),
      /"slow parity","passed","5000"/,
    );
    assert.match(
      fs.readFileSync(path.join(outputDir, "native-api-tests-summary.md"), "utf8"),
      /\| 5000\.0 \| passed \| slow parity \|/,
    );
  } finally {
    fs.rmSync(outputDir, { recursive: true, force: true });
  }
});

test("missing measurements and empty reports fail explicitly", () => {
  assert.throws(
    () => summarize(fixture("missing-duration")),
    /Completed test has no finite duration/,
  );
  assert.throws(
    () => summarize(fixture("empty-duration")),
    /Completed test has no finite duration/,
  );
  assert.throws(() => summarize(fixture("empty")), /no testResults/);
});

test("a failing test remains visible without triggering the passed-test timing gate", () => {
  const result = summarize(fixture("failed"));
  assert.equal(result.slowPassed.length, 0);
  assert.equal(result.ranked[0].status, "failed");
  assert.equal(result.ranked[0].durationMs, 20009);
  assert.match(markdown(result), /\| 20009\.0 \| failed \| timed out SQL verdict \|/);
});
