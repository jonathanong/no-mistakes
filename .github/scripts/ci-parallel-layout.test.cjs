const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const root = path.resolve(__dirname, "../..");
const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
const job = (name) => workflow.split(`\n  ${name}:\n`)[1]?.split(/\n  [a-z][a-z-]*:\n/)[0];
const results = JSON.parse(
  readFileSync(path.join(root, "test-cases/ci-speed/fixture/aggregate-results.json"), "utf8"),
);

for (const name of [
  "quality",
  "native-tests",
  "javascript-tests",
  "tests-standalone-lib",
  "docs-tests",
  "tests-gate",
]) {
  test(`${name} cannot pass when a dependency fails, cancels, or skips`, () => {
    const body = job(name);
    assert.ok(body);
    assert.match(body, /if: \$\{\{ always\(\) \}\}/);
    const needs = body
      .match(/needs: \[([^\]]+)\]/)[1]
      .split(",")
      .map((value) => value.trim());
    const script = body.split("        run: |\n")[1].replace(/^          /gm, "");
    for (const dependency of needs) {
      for (const fixture of results) {
        const input = Object.fromEntries(needs.map((id) => [id, { result: "success" }]));
        input[dependency].result = fixture.result;
        const child = spawnSync("bash", ["-eu", "-c", script], {
          encoding: "utf8",
          env: { ...process.env, RESULTS: JSON.stringify(input) },
        });
        assert.equal(
          child.status,
          fixture.exit,
          `${name}/${dependency}/${fixture.result}: ${child.stderr}`,
        );
      }
    }
  });
}

test("required names and every full test family remain in the aggregate", () => {
  assert.match(job("tests-gate"), /name: Tests with coverage\n/);
  assert.match(job("ast-analysis"), /name: AST analysis\n/);
  for (const name of [
    "quality",
    "packages",
    "javascript-tests",
    "native-tests",
    "tests",
    "tests-standalone-lib",
    "timeout-policy-browser-tests",
  ])
    assert.ok(job("tests-gate").includes(name), name);
  for (const name of [
    "rust-coverage-build",
    "rust-coverage-shards",
    "standalone-build",
    "standalone-shards",
    "native-build",
    "native-test-build",
    "native-rust-tests",
    "native-api-tests",
    "javascript-shards",
  ])
    assert.doesNotMatch(job(name), /docs-only.*!= 'true'/);
});

test("JavaScript shards merge coverage under the normal unchanged threshold configuration", () => {
  assert.match(
    job("javascript-shards"),
    /--config \.github\/scripts\/vitest-shard.config.cjs --shard=/,
  );
  assert.match(job("javascript-shards"), /reporter=blob/);
  assert.match(job("javascript-shards"), /runs-on: ubuntu-latest/);
  assert.match(job("javascript-shards"), /include-hidden-files: true/);
  assert.match(job("javascript-shards"), /path: \.vitest\/blob/);
  assert.match(job("javascript-coverage"), /path: \.vitest\/blob/);
  assert.match(job("javascript-coverage"), /vitest --merge-reports --coverage/);
  assert.doesNotMatch(job("javascript-coverage"), /vitest-shard.config/);
  assert.match(job("javascript-coverage"), /needs.javascript-shards.result/);
});
