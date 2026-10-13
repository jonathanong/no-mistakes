const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { readFileSync } = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");

const root = path.resolve(__dirname, "../..");
const fixture = path.join(root, "test-cases/rust-validation/fixture/bin");
const run = (args, status = "0") =>
  spawnSync("bash", [".github/scripts/rust-validation.sh", ...args], {
    cwd: root,
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${fixture}${path.delimiter}${process.env.PATH}`,
      RUST_VALIDATION_FIXTURE_STATUS: status,
    },
  });

for (const [mode, targets] of Object.entries({
  build: ["--lib", "--bins", "--tests", "--no-run"],
  test: ["--lib", "--bins", "--tests"],
  doc: ["--doc"],
})) {
  test(`${mode} uses the locked all-feature workspace scope`, () => {
    const result = run([mode]);
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(result.stdout.trim().split("\n"), [
      "test",
      "--locked",
      "--workspace",
      "--all-features",
      ...targets,
    ]);
  });
}
test("invalid modes and additional arguments fail before invoking cargo", () => {
  for (const args of [[], ["other"], ["test", "--release"]]) {
    const result = run(args);
    assert.equal(result.status, 2);
    assert.equal(result.stdout, "");
  }
});
test("cargo failures propagate", () => assert.equal(run(["test"], "17").status, 17));
test("development and test profiles omit debugger symbols by default", () => {
  const manifest = readFileSync(path.join(root, "Cargo.toml"), "utf8");
  for (const profile of ["dev", "test"]) {
    const block = manifest.split(`[profile.${profile}]`)[1]?.split("[")[0];
    assert.match(block ?? "", /^debug = 0$/m);
  }
});
test("CI and release both use the shared test and doctest scopes", () => {
  for (const [file, job] of [
    ["ci.yml", "tests"],
    ["release.yml", "validate-rust"],
  ]) {
    const workflow = readFileSync(path.join(root, ".github/workflows", file), "utf8");
    const body = workflow.split(`\n  ${job}:\n`)[1]?.split(/\n  [a-z][a-z-]*:\n/)[0];
    assert.ok(body, `${file} must retain ${job}`);
    for (const mode of ["build", "test", "doc"]) {
      assert.match(
        body,
        new RegExp(`bash \\.github/scripts/rust-validation\\.sh ${mode}(?:\\n|$)`),
      );
    }
    assert.match(workflow, /node --test \.github\/scripts\/rust-validation\.test\.cjs/);
    assert.doesNotMatch(workflow, /CARGO_PROFILE_(?:DEV|TEST)_DEBUG:/);
  }
});

test("CI, release, and pre-push Clippy validate the locked all-feature workspace", () => {
  for (const file of ["ci.yml", "release.yml"]) {
    const workflow = readFileSync(path.join(root, ".github/workflows", file), "utf8");
    assert.match(
      workflow,
      /run: cargo clippy --locked --workspace --all-targets --all-features -- -D warnings/,
    );
  }
});

test("pre-push Clippy uses the workflow scope", () => {
  const hook = readFileSync(path.join(root, ".husky/pre-push"), "utf8");
  assert.match(
    hook,
    /cargo clippy --locked --workspace --all-targets --all-features -- -D warnings/,
  );
});
