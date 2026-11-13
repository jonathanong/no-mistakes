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

test("documentation tests cannot delay coverage archive consumers", () => {
  assert.match(job("docs-tests"), /bash \.github\/scripts\/rust-validation\.sh doc/);
  assert.match(job("docs-tests"), /shared-key: rust-doctests-v1/);
  assert.doesNotMatch(job("rust-coverage-build"), /rust-validation\.sh doc/);
  assert.doesNotMatch(job("rust-coverage-shards"), /needs:.*docs-tests/);
  assert.match(job("tests-gate"), /docs-tests/);
});

test("native workspace caches preserve content-verified compiler inputs as one artifact", () => {
  for (const id of ["native-build", "native-test-build"]) {
    const body = job(id);
    assert.ok(body.indexOf("id: native-rust-cache") < body.indexOf("id: native-workspace-cache"));
    assert.match(body, /cache-workspace-crates: false/);
    assert.match(body, /cache-on-failure: false/);
    assert.match(body, /uses: actions\/cache@[a-f0-9]{40}/);
    assert.match(body, /crates\/\*\*\/\*\.rs/);
    assert.match(body, /target\/debug/);
    assert.match(body, /runner\.temp \}\}\/nm-cargo-inputs/);
    const key = body.match(/^          key: ([^\n]+)/m)[1];
    const fallback = body.match(/restore-keys: \|\n            ([^\n]+)/)[1];
    assert.equal(key, fallback + "${{ hashFiles('crates/**/*.rs') }}");
    assert.ok(fallback.includes("steps.native-rust-toolchain.outputs.cachekey"));
    assert.ok(fallback.includes("env.RUSTFLAGS"));
    assert.ok(fallback.includes("hashFiles('Cargo.toml'"));
    const restore = body.indexOf("cargo-cache-inputs.cjs restore");
    const compile = body.indexOf(
      id === "native-build"
        ? "node .github/scripts/build-native.cjs"
        : "node .github/scripts/native-test-archive.cjs build",
    );
    const capture = body.indexOf("cargo-cache-inputs.cjs capture");
    assert.ok(restore > 0 && restore < compile && compile < capture);
    assert.match(body, /steps\.native-workspace-cache\.outputs\.cache-hit/);
  }
});
