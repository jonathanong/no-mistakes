const { workflowJob } = require("./workflow-contract.cjs");
const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const root = path.resolve(__dirname, "../..");
const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
const job = (name) => workflowJob(workflow, name);
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
      .match(/needs:\s*\[([^\]]+)\]/)[1]
      .split(",")
      .map((value) => value.trim())
      .filter(Boolean);
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
    "native-build-macos",
    "native-test-build",
    "native-test-build-macos",
    "native-rust-tests",
    "native-rust-tests-macos",
    "native-api-tests",
    "native-api-tests-macos",
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
  for (const id of [
    "native-build",
    "native-build-macos",
    "native-test-build",
    "native-test-build-macos",
  ]) {
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
      id.startsWith("native-build")
        ? "node .github/scripts/build-native.cjs"
        : "node .github/scripts/native-test-archive.cjs build",
    );
    const capture = body.indexOf("cargo-cache-inputs.cjs capture");
    assert.ok(restore > 0 && restore < compile && compile < capture);
    assert.match(body, /steps\.native-workspace-cache\.outputs\.cache-hit/);
  }
});

test("Linux CLI and addon producers preserve their commands and publish separate artifacts", () => {
  const body = job("linux-runtime");
  assert.match(body, /fail-fast: false/);
  assert.match(body, /role: cli\s+binary: target\/debug\/no-mistakes\s/);
  assert.match(body, /role: addon\s+binary: target\/debug\/no-mistakes\.node\s/);
  const step = (name) => body.split(`- name: ${name}\n`)[1]?.split(/\n      - name:/)[0];
  const cli = step("Build Linux CLI");
  const addon = step("Build Linux N-API addon");
  assert.match(cli, /if: \$\{\{ matrix.role == 'cli' \}\}/);
  assert.match(cli, /run: cargo build --locked -p no-mistakes --bin no-mistakes\s*$/);
  assert.match(addon, /if: \$\{\{ matrix.role == 'addon' \}\}/);
  assert.match(
    addon,
    /NO_MISTAKES_BUILD_NAPI=1 cargo rustc --locked -p no-mistakes --lib --crate-type cdylib\s+cp target\/debug\/libno_mistakes\.so target\/debug\/no-mistakes\.node/,
  );
  assert.equal([...body.matchAll(/NO_MISTAKES_BUILD_NAPI/g)].length, 1);
  assert.doesNotMatch(body, /--all-features|--no-default-features|--release|crate-type =/);
  assert.match(body, /name: linux-runtime-\$\{\{ matrix.role \}\}/);
  assert.match(body, /path: \$\{\{ matrix.binary \}\}/);
  assert.equal([...body.matchAll(/uses: actions\/upload-artifact@/g)].length, 1);
  assert.match(job("linux-api-tests"), /needs: linux-runtime/);
  assert.match(job("linux-api-tests"), /pattern: linux-runtime-\*/);
  assert.match(job("linux-api-tests"), /merge-multiple: true/);
  assert.match(job("ast-analysis"), /name: linux-runtime-cli/);
  assert.doesNotMatch(job("ast-analysis"), /pattern: linux-runtime|name: linux-runtime-addon/);
});

test("Linux role caches restore content sidecars after shared dependencies and before compilation", () => {
  const body = job("linux-runtime");
  const dependency = body.indexOf("shared-key: linux-runtime-v1");
  const workspace = body.indexOf("id: linux-workspace-cache");
  const restore = body.indexOf("cargo-cache-inputs.cjs restore");
  const cli = body.indexOf("- name: Build Linux CLI");
  const addon = body.indexOf("- name: Build Linux N-API addon");
  const capture = body.indexOf("cargo-cache-inputs.cjs capture");
  assert.ok(
    dependency < workspace &&
      workspace < restore &&
      restore < cli &&
      cli < addon &&
      addon < capture,
  );
  assert.match(body, /cache-workspace-crates: false/);
  assert.match(body, /cache-on-failure: false/);
  assert.match(body, /workspaces: \. -> target/);
  assert.match(
    body,
    /target\/debug\s+target\/\.rustc_info\.json\s+\$\{\{ runner.temp \}\}\/linux-cargo-inputs/,
  );
  const key = body.match(/^          key: ([^\n]+)/m)[1];
  const fallback = body.match(/restore-keys: \|\n            ([^\n]+)/)[1];
  assert.equal(key, fallback + "${{ hashFiles('crates/**/*.rs') }}");
  for (const part of [
    "matrix.role",
    "steps.linux-rust-toolchain.outputs.cachekey",
    "env.RUSTFLAGS",
    "'Cargo.toml'",
    "'Cargo.lock'",
    "'crates/**/Cargo.toml'",
  ])
    assert.ok(fallback.includes(part), `Linux cache prefix must retain ${part}`);
});
