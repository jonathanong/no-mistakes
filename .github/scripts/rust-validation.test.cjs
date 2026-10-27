const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { readFileSync, existsSync, cpSync, mkdtempSync, renameSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const { unreferencedSnapshots, snapshotFiles } = require("./rust-snapshot-references.cjs");

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
test("orphan snapshot audit failures propagate after successful tests", () => {
  const result = spawnSync("bash", [".github/scripts/rust-validation.sh", "test"], {
    cwd: root,
    encoding: "utf8",
    env: {
      ...process.env,
      PATH: `${fixture}${path.delimiter}${process.env.PATH}`,
      RUST_VALIDATION_FIXTURE_STATUS: "0",
      RUST_VALIDATION_FIXTURE_SNAPSHOT_STATUS: "19",
    },
  });
  assert.equal(result.status, 19, result.stderr);
});

const snapshotFixture = path.join(root, "test-cases/rust-snapshot-references/fixture");
const snapshotCases = JSON.parse(readFileSync(path.join(snapshotFixture, "cases.json"), "utf8"));
for (const scenario of snapshotCases) {
  test(`snapshot references: ${scenario.name}`, () => {
    const absolute = (files) => files.map((file) => path.join(snapshotFixture, file));
    assert.deepEqual(
      unreferencedSnapshots(absolute(scenario.files), absolute(scenario.references), [
        path.join(snapshotFixture, "member"),
      ]),
      absolute(scenario.expected),
    );
  });
}
test("release archives retain orphan auditing and ignore generated snapshots", () => {
  const archive = mkdtempSync(path.join(tmpdir(), "rust-snapshot-archive-"));
  try {
    cpSync(snapshotFixture, archive, { recursive: true });
    renameSync(path.join(archive, ".gitignore.fixture"), path.join(archive, ".gitignore"));
    renameSync(path.join(archive, "ignored.snap.fixture"), path.join(archive, "ignored.snap"));
    const files = snapshotFiles(archive);
    const relative = (items) =>
      items.map((file) => path.relative(archive, file).split(path.sep).join("/")).sort();
    assert.deepEqual(relative(files), [
      "member/snapshots/orphan.snap",
      "member/snapshots/used.snap",
      "outside/snapshots/fixture.snap",
    ]);
    assert.deepEqual(
      relative(
        unreferencedSnapshots(
          files,
          [path.join(archive, "member/snapshots/used.snap")],
          [path.join(archive, "member")],
        ),
      ),
      ["member/snapshots/orphan.snap"],
    );
    const audit = (references) =>
      spawnSync(
        process.execPath,
        [path.join(__dirname, "rust-snapshot-references.cjs"), path.join(archive, references)],
        { cwd: archive, encoding: "utf8" },
      );
    const orphan = audit("orphan-references.txt");
    assert.equal(orphan.status, 1, orphan.stderr);
    assert.match(
      orphan.stderr,
      /Unreferenced Rust snapshots[\s\S]*member\/snapshots\/orphan\.snap/,
    );
    const complete = audit("all-references.txt");
    assert.equal(complete.status, 0, complete.stderr);
    assert.equal(existsSync(path.join(archive, ".git")), false);
  } finally {
    rmSync(archive, { recursive: true, force: true });
  }
});
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
    const prefix = job === "validate-rust" ? '"$RUNNER_TEMP/validation-tools/' : "";
    const quote = job === "validate-rust" ? '"' : "";
    for (const mode of ["build", "test", "doc"]) {
      assert.ok(
        body.includes(`bash ${prefix}.github/scripts/rust-validation.sh${quote} ${mode}\n`),
        `${file} must use the shared ${mode} scope`,
      );
    }
    assert.ok(
      workflow.includes(`node --test ${prefix}.github/scripts/rust-validation.test.cjs${quote}`),
    );
    const overrides = [...workflow.matchAll(/CARGO_PROFILE_(?:DEV|TEST)_DEBUG: (.+)/g)];
    for (const [, value] of overrides) assert.equal(value, '"0"');
    if (job === "tests") assert.equal(overrides.length, 0);
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

test("CI Clippy also builds default-feature test targets", () => {
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  assert.match(
    workflow,
    /run: cargo clippy --locked -p no-mistakes --all-targets -- -D warnings\n/,
  );
});

test("pre-push Clippy uses the workflow scope", () => {
  const hook = readFileSync(path.join(root, ".husky/pre-push"), "utf8");
  assert.match(
    hook,
    /cargo clippy --locked --workspace --all-targets --all-features -- -D warnings/,
  );
});

test("release validation tools come from the workflow without replacing tagged Rust source", () => {
  const workflow = readFileSync(path.join(root, ".github/workflows/release.yml"), "utf8");
  const body = workflow.split("\n  validate-rust:\n")[1].split(/\n  [a-z][a-z-]*:\n/)[0];
  const checkout = body
    .split("- name: Checkout validation tooling\n")[1]
    ?.split("\n      - name:")[0];
  assert.ok(checkout);
  assert.match(checkout, /ref: \$\{\{ github\.workflow_sha \}\}/);
  assert.match(checkout, /path: validation-tools/);
  assert.match(checkout, /persist-credentials: false/);
  assert.match(checkout, /fetch-depth: 1/);
  assert.doesNotMatch(checkout, /sparse-checkout/);
  assert.doesNotMatch(body, /working-directory:/);
  for (const profile of ["DEV", "TEST"])
    assert.match(body, new RegExp(`CARGO_PROFILE_${profile}_DEBUG: "0"`));
});

test("release isolates workflow tooling before validating tagged source", () => {
  const workflow = readFileSync(path.join(root, ".github/workflows/release.yml"), "utf8");
  const body = workflow.split("\n  validate-rust:\n")[1].split(/\n  [a-z][a-z-]*:\n/)[0];
  const checkout = body.indexOf("- name: Checkout validation tooling\n");
  const isolation = body.indexOf("- name: Isolate validation tooling\n");
  const validation = body.indexOf("- name: Check Rust validation parity\n");
  assert.ok(checkout >= 0 && isolation > checkout && validation > isolation);
  assert.match(body, /run: mv -- validation-tools "\$RUNNER_TEMP\/validation-tools"/);
  assert.doesNotMatch(body, /run: (?:bash|node --test) validation-tools\//);
});

test("shared tooling validates a historical release directory without local helpers", () => {
  const historical = path.join(root, "test-cases/rust-validation/fixture/historical-release");
  assert.equal(existsSync(path.join(historical, ".github/scripts/rust-validation.sh")), false);
  const result = spawnSync(
    "bash",
    [path.join(root, ".github/scripts/rust-validation.sh"), "test"],
    {
      cwd: historical,
      encoding: "utf8",
      env: {
        ...process.env,
        PATH: `${fixture}${path.delimiter}${process.env.PATH}`,
        RUST_VALIDATION_FIXTURE_PRINT_CWD: "1",
        RUST_VALIDATION_FIXTURE_STATUS: "0",
      },
    },
  );
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout.trim().split("\n")[0], historical);
});

test("pre-push enforces the same documentation spelling scope as CI", () => {
  const hook = readFileSync(path.join(root, ".husky/pre-push"), "utf8");
  assert.match(hook, /pnpm run lint:spell/);
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  assert.match(workflow, /pnpm run lint:spell/);
  assert.match(hook, /wait "\$pid_cspell" \|\| fail=1/);
});

test("native API timeout leaves startup and teardown room after the observed Windows suite", () => {
  const observed = JSON.parse(
    readFileSync(
      path.join(root, "test-cases/rust-validation/fixture/windows-native-api-timing.json"),
      "utf8",
    ),
  );
  const duration = Number(observed.observedLogLines.join("\n").match(/Duration\s+(\d+\.\d+)s/)[1]);
  assert.match(observed.observedLogLines.join("\n"), /252 passed.*2 skipped/);
  assert.equal(observed.step.conclusion, "failure");
  const required = duration + observed.startupTeardownReserveSeconds;
  assert.ok(required > 120, "the observed suite cannot safely fit the former two-minute step");
  const workflow = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  const native = workflow.split("\n  native-tests:\n")[1].split(/\n  [a-z][a-z-]*:\n/)[0];
  const minutes = Number(
    native.match(/name: Run real N-API API test\n\s+timeout-minutes: (\d+)/)[1],
  );
  assert.ok(minutes * 60 >= required, "native API step must include launcher/teardown reserve");
  assert.ok(minutes <= 5, "keep the correction bounded rather than removing its deadline");
  assert.match(native, /\n    timeout-minutes: 20\n/);
  const linux = workflow.split("\n  native-tests:\n")[0];
  assert.match(linux, /name: Run real N-API API test\n\s+timeout-minutes: 2\n/);
});
