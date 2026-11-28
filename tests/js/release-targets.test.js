const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const { isMap, isSeq, parseDocument } = require("yaml");

const { readWorkflow } = require("../../.github/scripts/workflow-contract.cjs");
const { nativePackageName } = require("../../packages/no-mistakes/scripts/native-package");

const repoRoot = join(__dirname, "..", "..");

test("oxlint turns max-lines off for JavaScript tests", () => {
  const config = JSON.parse(readFileSync(join(repoRoot, ".oxlintrc.json"), "utf8"));
  const tests = config.overrides.find((override) => override.files?.includes("**/*.test.js"));
  assert.equal(tests?.rules?.["max-lines"], "off");
});

function jobBody(source, name) {
  const parts = source.split(`\n  ${name}:\n`);
  assert.equal(parts.length, 2, `release workflow must define ${name}`);
  return parts[1].split(/\n  [\w-]+:\n/)[0];
}

function releaseBuildTargets(source) {
  return [
    "build-native-linux-x64",
    "build-native-linux-arm",
    "build-native-darwin",
    "build-native-windows",
  ].map((name) => {
    const target = jobBody(source, name).match(/^ {6}NM_TARGET: (\S+)$/m);
    assert.ok(target, `${name} must name its Cargo target`);
    return target[1];
  });
}

test("every native optional package has a release target", () => {
  const releaseWorkflow = readFileSync(
    join(repoRoot, ".github", "workflows", "release.yml"),
    "utf8",
  );
  const releaseTargets = releaseBuildTargets(releaseWorkflow);
  const targets = [
    ["darwin", "arm64", "aarch64-apple-darwin"],
    ["win32", "x64", "x86_64-pc-windows-msvc"],
    ["linux", "x64", "x86_64-unknown-linux-gnu"],
    ["linux", "arm64", "aarch64-unknown-linux-gnu"],
  ];

  for (const [platform, arch, target] of targets) {
    assert.match(nativePackageName(platform, arch), /^no-mistakes-/);
    assert.ok(releaseTargets.includes(target), `release workflow does not build ${target}`);
  }
});

test("release jobs declare pinned action uses outside shared steps", () => {
  const releasePath = join(repoRoot, ".github/workflows/release.yml");
  const workflow = parseDocument(readFileSync(releasePath, "utf8"));
  const { jobs } = readWorkflow(releasePath);
  for (const [name, job] of Object.entries(jobs)) {
    const actions = job.steps?.filter((step) => step.uses) ?? [];
    if (actions.length === 0) continue;
    const steps = workflow.getIn(["jobs", name, "steps"], true);
    assert.ok(isSeq(steps), name);
    const directActions = steps.items.filter((step) => isMap(step) && step.has("uses"));
    assert.equal(directActions.length, actions.length, name);
  }
});

test("release native builds emit CLI and N-API from one rustc without waiting for Validate", () => {
  const { jobs } = readWorkflow(join(repoRoot, ".github/workflows/release.yml"));
  const script = readFileSync(join(repoRoot, ".github/scripts/cargo-release-build.sh"), "utf8");
  assert.match(script, /crate-type = \["rlib", "cdylib"\]/);
  assert.match(
    script,
    /cargo build --release --locked --target "\$target" -p no-mistakes --timings/,
  );
  assert.doesNotMatch(script, /--crate-type cdylib/);
  assert.match(script, /target\/cargo-timings\/cargo-timing-\*\.html/);

  for (const [target, jobTimeout, buildTimeout, addon] of [
    ["linux-x64", 100, "65", "libno_mistakes.so"],
    ["linux-arm", 100, "65", "libno_mistakes.so"],
    ["darwin", 120, "90", "libno_mistakes.dylib"],
    ["windows", 120, "90", "no_mistakes.dll"],
  ]) {
    const { needs, env, steps, "timeout-minutes": timeout } = jobs[`build-native-${target}`];
    assert.deepEqual(needs, ["prepare"]);
    assert.equal(timeout, jobTimeout);
    assert.equal(env.NM_BUILD_TIMEOUT, buildTimeout);
    assert.equal(env.NM_ADDON, addon);

    const step = (name) => steps.find((entry) => entry.name === name);
    const compile = step("Build CLI and N-API addon");
    assert.equal(compile["timeout-minutes"], "${{ fromJSON(env.NM_BUILD_TIMEOUT) }}");
    assert.ok(
      steps
        .filter((entry) => entry !== compile)
        .every(
          (entry) => Number.isInteger(entry["timeout-minutes"]) && entry["timeout-minutes"] <= 5,
        ),
    );
    assert.match(
      compile.run,
      /bash "\$RUNNER_TEMP\/release-scripts\/\.github\/scripts\/cargo-release-build\.sh"/,
    );
    assert.equal(step("Checkout release scripts").with.ref, "${{ github.workflow_sha }}");
    const isolate = step("Isolate release scripts");
    assert.equal(isolate.shell, "bash");
    assert.equal(isolate.run, 'mv -- release-scripts "$RUNNER_TEMP/release-scripts"');
    assert.match(
      step("Exclude workspace from Microsoft Defender").run,
      /Add-MpPreference -ExclusionPath/,
    );
    assert.match(step("Cache Rust build artifacts").uses, /^Swatinem\/rust-cache@[a-f0-9]{40}$/);

    const required = step("Require Cargo timings report");
    const timings = step("Upload Cargo timings");
    assert.ok(steps.indexOf(required) < steps.indexOf(timings));
    assert.equal(required["timeout-minutes"], 1);
    assert.equal(required["continue-on-error"], undefined);
    assert.match(required.run, /target\/cargo-timings\/\*\.html/);
    assert.equal(timings["continue-on-error"], true);
    assert.equal(timings.with.path, "target/cargo-timings/*.html");
    assert.equal(timings.with["if-no-files-found"], "error");

    for (const [name, artifact] of [
      ["Upload CLI artifact", "release-cli"],
      ["Upload N-API artifact", "release-napi"],
      ["Upload Cargo timings", "release-timings"],
    ]) {
      const upload = step(name);
      assert.equal(upload.with.name, artifact + "-${{ env.NM_TARGET }}");
      if (upload !== timings) assert.equal(upload["continue-on-error"], undefined);
    }
  }
});

test("release syncs optional native package versions and publishes only through npm OIDC", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  const publisher = readFileSync(join(repoRoot, ".github/scripts/publish-one-npm.sh"), "utf8");
  const stager = readFileSync(join(repoRoot, ".github/scripts/stage-release-addon.sh"), "utf8");
  assert.match(workflow, /sync-native-package-versions\.js "\$version"/);
  assert.match(publisher, /publish-npm\.cjs" "\$package" "\$version"/);
  const helper = readFileSync(join(repoRoot, ".github/scripts/publish-npm.cjs"), "utf8");
  assert.match(helper, /"--provenance", "--access", "public", "--tag"/);
  assert.doesNotMatch(workflow, /NPM_TOKEN|pnpm[^\n]* publish/);
  const publish = jobBody(workflow, "publish-native-linux-x64");
  const publishJs = jobBody(workflow, "publish-js");
  for (const name of [
    "no-mistakes-darwin-arm64",
    "no-mistakes-linux-arm64-gnu",
    "no-mistakes-linux-x64-gnu",
    "no-mistakes-win32-x64-msvc",
  ]) {
    assert.match(workflow, new RegExp(`NM_PACKAGE: ${name}\\n`));
    assert.doesNotMatch(publishJs, new RegExp(name));
  }
  assert.match(
    publishJs,
    /publish-tools\/\.github\/scripts\/publish-one-npm\.sh" "\$\{\{ needs\.prepare\.outputs\.version \}\}" no-mistakes/,
  );
  for (const verify of [
    "verify-npm-darwin",
    "verify-npm-linux-x64",
    "verify-npm-linux-arm",
    "verify-npm-windows",
  ]) {
    assert.match(publishJs, new RegExp(`^ {6}- ${verify}$`, "m"));
  }
  assert.match(publish, /needs:\n {6}- prepare\n {6}- validate\n {6}- build-native-linux-x64/);
  assert.doesNotMatch(publish, /build-native-darwin/);
  assert.match(stager, /addon="target\/\$\{target\}\/release\/\$\{addon_name\}"/);
  assert.match(stager, /expected_magic='Mach-O\.\*arm64'/);
  assert.match(stager, /expected_magic='ELF 64-bit\.\*x86-64'/);
  assert.match(stager, /expected_magic='PE32\\\+\.\*x86-64'/);
});

test("Cargo publication overlaps the native and npm path after validation", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  const cargo = jobBody(workflow, "publish-cargo");
  const npm = jobBody(workflow, "publish-native-linux-x64");
  assert.match(cargo, /^ {4}needs:\n {6}- prepare\n {6}- validate$/m);
  assert.match(cargo, /ref: \$\{\{ needs\.prepare\.outputs\.release-sha \}\}/);
  assert.match(cargo, /persist-credentials: false/);
  assert.match(cargo, /cargo publish --locked -p "\$pkg"/);
  assert.doesNotMatch(cargo, /--no-verify|id-token: write|contents: write|RELEASE_TOKEN/);
  assert.match(cargo, /already exists on crates\.io index/);
  assert.match(cargo, /crates\.io\/api\/v1\/crates\/\$pkg\/\$version/);
  assert.doesNotMatch(npm, /Publish Cargo crates|Install Rust|CARGO_REGISTRY_TOKEN|publish-cargo/);
  assert.match(npm, /^ {4}needs:\n {6}- prepare\n {6}- validate\n {6}- build-native-linux-x64$/m);
});

test("ESLint publication waits for Linux and Windows root verification only", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  const eslint = jobBody(workflow, "publish-eslint");
  const root = jobBody(workflow, "publish-js");
  const verifyMac = jobBody(workflow, "verify-npm-root-darwin");
  assert.match(
    eslint,
    /^ {4}needs:\n {6}- prepare\n {6}- verify-npm-root-linux-x64\n {6}- verify-npm-root-linux-arm\n {6}- verify-npm-root-windows$/m,
  );
  assert.doesNotMatch(eslint, /verify-npm-root-darwin/);
  assert.match(eslint, /ref: \$\{\{ needs\.prepare\.outputs\.release-sha \}\}/);
  assert.match(eslint, /^ {6}contents: read$/m);
  assert.match(eslint, /^ {6}id-token: write$/m);
  assert.match(eslint, /persist-credentials: false/);
  assert.doesNotMatch(eslint, /RELEASE_TOKEN|NPM_TOKEN/);
  assert.match(eslint, /eslint-plugin-no-mistakes/);
  assert.doesNotMatch(eslint, /publish-one-npm\.sh[^\n]* no-mistakes$/m);
  for (const verify of [
    "verify-npm-darwin",
    "verify-npm-linux-x64",
    "verify-npm-linux-arm",
    "verify-npm-windows",
  ]) {
    assert.match(root, new RegExp(`^ {6}- ${verify}$`, "m"));
  }
  assert.match(verifyMac, /^ {6}- publish-js$/m);
  assert.doesNotMatch(root, /publish-eslint/);
  assert.doesNotMatch(verifyMac, /^ {6}- publish-eslint$/m);
  const complete = jobBody(workflow, "release-complete");
  assert.match(complete, /^ {6}- publish-eslint$/m);
  assert.match(complete, /^ {6}- verify-npm-root-darwin$/m);
  assert.match(complete, /^ {4}runs-on: ubuntu-slim$/m);
});

test("release verifies each published package on its target platform", () => {
  const { jobs } = readWorkflow(join(repoRoot, ".github/workflows/release.yml"));
  for (const [id, os, packageName] of [
    ["verify-npm-darwin", "macos-15", "no-mistakes-darwin-arm64"],
    ["verify-npm-linux-x64", "ubuntu-22.04", "no-mistakes-linux-x64-gnu"],
    ["verify-npm-linux-arm", "ubuntu-22.04-arm", "no-mistakes-linux-arm64-gnu"],
    ["verify-npm-windows", "windows-2025", "no-mistakes-win32-x64-msvc"],
    ["verify-npm-root-linux-x64", "ubuntu-22.04", "no-mistakes"],
  ]) {
    const job = jobs[id];
    assert.equal(job["runs-on"], os);
    assert.equal(job.env.NM_PACKAGE, packageName);
    const checkout = job.steps.find((step) => step.name === "Checkout release commit");
    assert.equal(checkout.with["persist-credentials"], false);
    const smoke = job.steps.find((step) => step.name === "Install and load published package");
    assert.match(smoke.run, /wait-npm-tarball\.js/);
    assert.match(smoke.run, /smoke-published-native\.js/);
  }
});

test("native build timings workflow compiles darwin and linux-x64 without publishing", () => {
  const workflow = readFileSync(
    join(repoRoot, ".github/workflows/native-build-timings.yml"),
    "utf8",
  );
  assert.match(workflow, /^permissions: \{\}$/m);
  assert.match(workflow, /CARGO_PROFILE_RELEASE_LTO: thin/);
  assert.match(workflow, /CARGO_PROFILE_RELEASE_CODEGEN_UNITS: "16"/);
  assert.match(workflow, /aarch64-apple-darwin/);
  assert.match(workflow, /x86_64-unknown-linux-gnu/);
  assert.doesNotMatch(workflow, /x86_64-pc-windows-msvc|aarch64-unknown-linux-gnu/);
  assert.match(workflow, /bash \.github\/scripts\/cargo-release-build\.sh/);
  assert.match(workflow, /name: release-timings-\$\{\{ matrix\.target \}\}/);
  assert.match(workflow, /path: target\/cargo-timings\/\*\.html/);
  assert.match(workflow, /if-no-files-found: error/);
  assert.doesNotMatch(workflow, /continue-on-error/);
  assert.doesNotMatch(workflow, /target\/cargo-timing\.html/);
  assert.match(workflow, /persist-credentials: false/);
  assert.match(workflow, /\n    timeout-minutes: 100\n/);
  assert.match(workflow, /\n        timeout-minutes: 90\n/);
  const onKeys = (source) => {
    const keys = [];
    for (const line of (source.split(/^on:\n/m)[1] ?? "").split("\n")) {
      if (line === "" || line.startsWith("#")) continue;
      if (line[0] !== " ") break;
      const key = /^ {2}(\S[^:]*):/.exec(line);
      if (key) keys.push(key[1]);
    }
    return keys;
  };
  assert.deepEqual(onKeys(workflow), ["workflow_dispatch"]);
  assert.deepEqual(
    onKeys('on:\n  workflow_dispatch:\n\n  schedule:\n    - cron: "0 0 * * *"\njobs:\n'),
    ["workflow_dispatch", "schedule"],
  );
  assert.doesNotMatch(workflow, /pull_request:|id-token:|publish-npm/);
});

test("CI cancels superseded PR runs but keeps active main coverage runs", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
  const concurrency = workflow.match(/^concurrency:\n((?: {2}[^\n]+\n)+)/m);
  assert.ok(concurrency, "CI must retain workflow-level concurrency");
  // PRs group per PR number, manual dispatches get their own per-run group so
  // pushes never drop a dispatch, and pushes group per ref so a started main
  // run completes while only the newest waiting main run is kept.
  assert.match(
    concurrency[1],
    /group: \$\{\{ github\.workflow \}\}-\$\{\{ github\.event_name == 'pull_request' && format\('pr-\{0\}', github\.event\.pull_request\.number\) \|\| \(github\.event_name == 'workflow_dispatch' && format\('dispatch-\{0\}', github\.run_id\) \|\| github\.ref\) \}\}\n/,
  );
  assert.match(
    concurrency[1],
    /github\.event_name == 'workflow_dispatch' && format\('dispatch-\{0\}', github\.run_id\)/,
  );
  assert.doesNotMatch(concurrency[1], /github\.sha/);
  assert.match(
    concurrency[1],
    /cancel-in-progress: \$\{\{ github\.event_name == 'pull_request' \}\}/,
  );
});

test("only Windows native timing breaches are report-only", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
  const reports = [
    ...workflow.matchAll(
      / {6}- name: Report real N-API test durations\n[\s\S]*?(?=\n {6}- name:)/g,
    ),
  ];
  assert.equal(reports.length, 3);
  assert.doesNotMatch(reports[0][0], /--report-only|continue-on-error/);
  for (const report of reports.slice(1)) {
    assert.match(report[0], /runner\.os == 'Windows' && '--report-only' \|\| ''/);
    assert.doesNotMatch(report[0], /continue-on-error/);
  }
});

test("native CI jobs run only platform-specific Rust tests", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
  const nativeJob = workflow.match(
    /^ {2}native-(?:build|test-build|rust-tests|api-tests):[\s\S]*?(?=^ {2}[a-z])/gm,
  );
  assert.ok(nativeJob, "ci.yml must define native-tests");
  const body = nativeJob.join("\n");

  // Bash `\` continuations are one argv to cargo; a newline-limited regex
  // would miss `--all-features` on the next physical line.
  const { commands } = require("../../.github/scripts/native-rust-tests.cjs");
  const nativeCommands = ["Windows", "macOS"]
    .flatMap((platform) =>
      ["build", "run"].flatMap((mode) =>
        commands(mode, platform).map((args) => `cargo ${args.join(" ")}`),
      ),
    )
    .join("\n");
  assert.match(body, /node \.github\/scripts\/native-test-archive.cjs run/);
  const unfoldedNativeJob = `${body.replace(/\\\r?\n[ \t]*/g, " ")}\n${nativeCommands}`;
  assert.doesNotMatch(
    unfoldedNativeJob,
    /cargo test\b[^\r\n]*--workspace\b/,
    "native jobs must not compile or run the Linux full suite",
  );
  assert.match(
    nativeCommands,
    /cargo test(?: --config \S+)* --locked -p no-mistakes-invocation --test macos_invocation/,
    "macOS native jobs must run the focused invocation test target",
  );
  assert.doesNotMatch(nativeCommands, /cargo test\b[^\r\n]*--lib\b/);
  assert.doesNotMatch(
    unfoldedNativeJob,
    /cargo test\b[^\r\n]*--all-features\b/,
    "native jobs must not pass --all-features to cargo test, including across bash line continuations",
  );
  assert.match(
    nativeCommands,
    /cargo test --locked -p no-mistakes-invocation --test windows_job_object/,
    "Windows must run the Job Object regression as an integration test",
  );
  assert.doesNotMatch(nativeCommands, /-p no-mistakes |suite_cli_|--bin/);
  const macosInvocation = readFileSync(
    join(repoRoot, "crates", "no-mistakes-invocation", "tests", "macos_invocation.rs"),
    "utf8",
  );
  assert.match(macosInvocation, /#\[path = "\.\.\/src\/invocation\.rs"\]/);
  assert.match(macosInvocation, /mod invocation;/);
  assert.doesNotMatch(
    body,
    /pull-requests:\s*write/,
    "native jobs must not request pull-requests write; timing stays in the job summary",
  );
  assert.doesNotMatch(
    body,
    /PR_NUMBER/,
    "native timing reports must not take a pull-request number for commenting",
  );
  assert.match(
    body,
    /head\.repo\.full_name == github\.repository/,
    "native timing reports must not run on fork PRs",
  );
  const defenderStep = body.match(
    /- name: Exclude workspace from Microsoft Defender[\s\S]*?(?=\n      - name: Checkout)/,
  );
  assert.ok(defenderStep, "Defender exclusion must run before Checkout");
  const defender = defenderStep[0];
  assert.match(defender, /runner\.os == 'Windows'/);
  assert.match(
    defender,
    /github\.event_name != 'pull_request' \|\| github\.event\.pull_request\.head\.repo\.full_name == github\.repository/,
    "Defender exclusions must not run on untrusted fork pull requests",
  );
  assert.match(defender, /Add-MpPreference -ExclusionPath/);
  assert.match(defender, /GITHUB_WORKSPACE/);
  assert.match(defender, /'\.cargo'/);
  assert.match(defender, /'\.rustup'/);
  assert.match(body, /Run native CLI smoke test/);
  assert.ok(
    require("../../.github/scripts/native-api-files.cjs").platformFiles.includes(
      "packages/no-mistakes/scripts/real-napi-api.test.js",
    ),
  );
  assert.match(
    body,
    /crate-type = \["rlib", "cdylib"\]/,
    "native jobs must emit rlib and cdylib from one rustc",
  );
  assert.doesNotMatch(
    body,
    /cargo rustc --locked -p no-mistakes --lib --crate-type cdylib/,
    "native jobs must not compile the crate a second time as cdylib-only",
  );
  assert.match(
    workflow,
    /node \.github\/scripts\/rust-coverage-shards\.cjs shard/,
    "Linux coverage executes the compiled full-suite archive through shard validation",
  );
});

test("both native API jobs run the compiled download-integrity regression when in scope", () => {
  const { jobs } = readWorkflow(join(repoRoot, ".github", "workflows", "ci.yml"));
  const scripts = join(repoRoot, "packages", "no-mistakes", "scripts");
  const umbrella = readFileSync(join(scripts, "real-napi-api.test.js"), "utf8");
  assert.match(umbrella, /require\("\.\/github-actions-download-integrity-api\.test\.js"\)/);

  for (const [name, platform] of [
    ["native-api-tests", "windows"],
    ["native-api-tests-macos", "macos"],
  ]) {
    const job = jobs[name];
    assert.equal(job.if, `\${{ needs.changes.outputs.${platform} == 'true' }}`);
    assert.equal(job.strategy.matrix.include.length, 1);
    assert.ok(jobs["tests-gate"].needs.includes(name));
    const run = job.steps.find((step) => step.id === "real-napi");
    assert.equal(run.if, undefined);
    assert.match(run.run, /node \.github\/scripts\/native-api-shards\.cjs/);
  }
});

// Keep native build/test debug settings aligned without relying on Cargo to
// reuse the preceding library build for integration tests.
test("native CI omits debug metadata without weakening runtime checks", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
  const body = workflow
    .match(/^ {2}native-(?:build|test-build|rust-tests|api-tests):[\s\S]*?(?=^ {2}[a-z])/gm)
    ?.join("\n");
  assert.ok(body, "ci.yml must define native-tests");
  const manifest = readFileSync(join(repoRoot, "Cargo.toml"), "utf8");
  for (const profile of ["dev", "test"]) {
    const settings = manifest.split(`[profile.${profile}]`)[1]?.split("[")[0];
    assert.match(settings ?? "", /^debug = 0$/m);
  }
  assert.doesNotMatch(body, /CARGO_PROFILE_(?:DEV|TEST)_DEBUG:/);
  assert.doesNotMatch(body, /CARGO_PROFILE_(?:DEV|TEST)_(?:DEBUG_ASSERTIONS|OVERFLOW_CHECKS):/);
  const stepEnvs = [...body.matchAll(/^ {8}env:\n((?:^ {10}.*\n)+)/gm)];
  for (const [, stepEnv] of stepEnvs) {
    assert.doesNotMatch(stepEnv, /CARGO_PROFILE_(?:DEV|TEST)_DEBUG:/);
  }
  const { ci: deadlines } = require("../../.github/scripts/fixtures/rust-build-deadlines.json");
  for (const [name, minutes] of [
    ["Build native CLI and N-API addon", 35],
    ["Precompile native Rust test targets", deadlines.testPrecompileStep / 60],
    ["Run platform-specific Rust tests", deadlines.platformRustStep / 60],
    ["Run real N-API API test", 10],
  ]) {
    const step = body.split(`- name: ${name}\n`)[1]?.split("\n      - name:")[0];
    assert.ok(step, `${name} must remain enabled`);
    assert.match(step, new RegExp(`^ {8}timeout-minutes: ${minutes}$`, "m"));
  }
});

// Publication must fail closed when either parallel suite fails or never runs.
test("release publication requires both parallel validation suites", () => {
  const { spawnSync } = require("node:child_process");
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  const job = (name) =>
    workflow.match(new RegExp(`^ {2}${name}:[\\s\\S]*?(?=^ {2}[a-z]|$(?![\\s\\S]))`, "m"))?.[0];
  const rust = job("validate-rust");
  const js = job("validate-js");
  const gate = job("validate");
  assert.ok(rust && js && gate);
  for (const suite of [rust, js]) {
    assert.match(suite, /^ {4}needs: prepare$/m);
    assert.match(suite, /name: release-source/);
    assert.match(suite, /tar -xzf prepared-release\/release-source.tgz/);
    assert.doesNotMatch(suite, /continue-on-error|npm publish|cargo publish --locked/);
  }
  const sampled = 'bash "$RUNNER_TEMP/validation-tools/.github/scripts/with-resource-samples.sh" ';
  for (const command of [
    "cargo fmt --all -- --check",
    sampled + "cargo clippy --locked --workspace --all-targets --all-features -- -D warnings",
    sampled + 'bash "$RUNNER_TEMP/validation-tools/.github/scripts/rust-validation.sh" build',
    sampled + 'bash "$RUNNER_TEMP/validation-tools/.github/scripts/rust-validation.sh" test-unit',
    sampled +
      'bash "$RUNNER_TEMP/validation-tools/.github/scripts/rust-validation.sh" test-integration',
    sampled + 'bash "$RUNNER_TEMP/validation-tools/.github/scripts/rust-validation.sh" doc',
    sampled + "cargo publish --dry-run --locked --workspace",
    "pnpm run build:js",
  ])
    assert.ok(rust.includes(`run: ${command}\n`), command);
  assert.match(rust, /shared-key: validate/);
  assert.match(rust, /uses: \.\/\.github\/actions\/start-test-postgres/);
  for (const command of [
    "pnpm run build:js",
    "pnpm run fmt:js:check",
    "pnpm run lint:js",
    "pnpm run lint:spell",
    "pnpm run test:ci",
    "pnpm run --if-present test:timeout-policy-browser",
  ]) {
    assert.ok(js.includes(`run: ${command}\n`), command);
  }
  assert.match(js, /playwright install chromium/);
  assert.doesNotMatch(js, /playwright install --with-deps/);
  assert.match(js, /pnpm --filter "\$pkg" pack --dry-run/);
  assert.doesNotMatch(js, /cargo|Install Rust/);
  assert.match(gate, /needs:\n {6}- validate-rust\n {6}- validate-js/);
  assert.match(gate, /if: \$\{\{ always\(\) \}\}/);
  assert.match(gate, /^ {4}runs-on: ubuntu-slim$/m);
  assert.match(gate, /permissions: \{\}/);
  assert.match(gate, /RUST_RESULT: \$\{\{ needs\.validate-rust\.result \}\}/);
  assert.match(gate, /JS_RESULT: \$\{\{ needs\.validate-js\.result \}\}/);
  const script = gate
    .split("run: |\n")[1]
    ?.trim()
    .replace(/^ {10}/gm, "");
  assert.ok(script);
  for (const rustResult of ["success", "failure", "cancelled", "skipped"]) {
    for (const jsResult of ["success", "failure", "cancelled", "skipped"]) {
      const result = spawnSync("bash", ["-e", "-c", script], {
        env: { ...process.env, RUST_RESULT: rustResult, JS_RESULT: jsResult },
      });
      assert.equal(result.status === 0, rustResult === "success" && jsResult === "success");
    }
  }
  for (const publisher of [
    "publish-cargo",
    "publish-github-release",
    "publish-native-linux-x64",
    "publish-native-linux-arm",
    "publish-native-darwin",
    "publish-native-windows",
  ]) {
    assert.match(job(publisher), /^ {6}- validate$/m);
  }
  const github = job("publish-github-release");
  assert.match(github, /::error::missing GitHub release asset/);
  for (const target of [
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
    "aarch64-apple-darwin",
    "x86_64-pc-windows-msvc",
  ]) {
    assert.match(github, new RegExp(target));
  }
});
