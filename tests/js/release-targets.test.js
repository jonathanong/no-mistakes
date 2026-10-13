const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const { nativePackageName } = require("../../packages/no-mistakes/scripts/native-package");

const repoRoot = join(__dirname, "..", "..");

function releaseMatrixTargets(source) {
  const match = source.match(/^ {6}matrix:\n {8}target:\n((?: {10}- [^\n]+\n)+)/m);
  assert.ok(match, "release workflow must define the build target matrix");
  return match[1]
    .trim()
    .split("\n")
    .map((line) => line.trim().replace(/^- /, ""));
}

test("every native optional package has a release target", () => {
  const releaseWorkflow = readFileSync(
    join(repoRoot, ".github", "workflows", "release.yml"),
    "utf8",
  );
  const releaseTargets = releaseMatrixTargets(releaseWorkflow);
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

test("release native builds emit CLI and N-API from one rustc without waiting for Validate", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  const job = workflow.match(/^ {2}build-native:[\s\S]*?(?=^ {2}publish:)/m);
  assert.ok(job, "release workflow must define build-native");
  const body = job[0];
  assert.match(body, /^ {4}needs:\n {6}- prepare$/m, "native builds must start after prepare");
  // Windows MSVC is the slowest, most variable, lowest-priority target, so it
  // alone gets extra headroom; every other target keeps 80/45 minutes.
  const windows = "matrix.target == 'x86_64-pc-windows-msvc'";
  assert.ok(
    body.includes(`\n    timeout-minutes: \${{ ${windows} && 100 || 80 }}\n`),
    "native builds keep an 80-minute job envelope (100 for Windows)",
  );
  assert.ok(
    body.includes(
      `\n      - name: Build CLI and N-API addon\n        timeout-minutes: \${{ ${windows} && 75 || 45 }}\n`,
    ),
    "combined compile keeps a 45-minute cold-build budget (75 for Windows)",
  );
  const stepTimeouts = [...body.matchAll(/^ {8}timeout-minutes: (\d+)$/gm)].map((timeout) =>
    Number(timeout[1]),
  );
  assert.ok(
    stepTimeouts.every((timeout) => timeout <= 45),
    "native steps must be at most 45 minutes",
  );
  assert.match(
    body,
    /crate-type = \["rlib", "cdylib"\]/,
    "native release jobs must emit rlib and cdylib from one rustc",
  );
  assert.doesNotMatch(
    body,
    /cargo rustc --release --locked --target .* --lib --crate-type cdylib/,
    "native release jobs must not compile the crate a second time as cdylib-only",
  );
  assert.match(body, /Add-MpPreference -ExclusionPath/);
  assert.match(body, /name: release-cli-\$\{\{ matrix\.target \}\}/);
  assert.match(body, /name: release-napi-\$\{\{ matrix\.target \}\}/);
  assert.match(body, /addon: libno_mistakes\.dylib/);
  assert.match(body, /addon: no_mistakes\.dll/);
});

test("release syncs optional native package versions and publishes only through npm OIDC", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  assert.match(workflow, /sync-native-package-versions\.js "\$version"/);
  assert.match(workflow, /npm publish "\.\/packages\/\$pkg" --provenance --access public/);
  assert.doesNotMatch(workflow, /NPM_TOKEN|pnpm[^\n]* publish/);
  const publish = workflow.match(/^ {2}publish:[\s\S]*?(?=^ {2}verify-npm-platform:)/m);
  const publishJs = workflow.match(/^ {2}publish-js:[\s\S]*?(?=^ {2}verify-npm-root:)/m);
  assert.ok(publish, "release workflow must define publish");
  assert.ok(publishJs, "release workflow must define publish-js");
  for (const name of [
    "no-mistakes-darwin-arm64",
    "no-mistakes-linux-arm64-gnu",
    "no-mistakes-linux-x64-gnu",
    "no-mistakes-win32-x64-msvc",
  ]) {
    assert.match(publish[0], new RegExp(`            ${name}(?: \\\\|$)`, "m"));
    assert.doesNotMatch(publishJs[0], new RegExp(`${name}`));
  }
  assert.match(publishJs[0], /^ {12}no-mistakes$/m);
  // Publish every package before waiting so one slow registry tarball cannot starve the rest.
  for (const job of [publish[0], publishJs[0]]) {
    const publishAt = job.indexOf('npm publish "./packages/$pkg"');
    const waitAt = job.indexOf('--version "$version" --timeout-ms 1800000');
    assert.ok(publishAt >= 0 && waitAt > publishAt, "publish all packages, then wait for each");
  }
  assert.match(publishJs[0], /needs:\n {6}- prepare\n {6}- verify-npm-platform/);
  assert.match(publish[0], /needs:\n {6}- prepare\n {6}- validate\n {6}- build-native/);
  assert.match(
    workflow,
    /addon="target\/\$\{\{ matrix\.target \}\}\/release\/\$\{\{ matrix\.addon \}\}"/,
  );
  assert.match(workflow, /expected_magic='Mach-O\.\*arm64'/);
  assert.match(workflow, /expected_magic='ELF 64-bit\.\*x86-64'/);
  assert.match(workflow, /expected_magic='PE32\\\+\.\*x86-64'/);
});

test("Cargo publication overlaps the native and npm path after validation", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  const cargo = workflow.match(/^ {2}publish-cargo:[\s\S]*?(?=^ {2}[a-z])/m)?.[0];
  const npm = workflow.match(/^ {2}publish:[\s\S]*?(?=^ {2}[a-z])/m)?.[0];
  assert.ok(cargo, "release must define independent Cargo publication");
  assert.ok(npm, "release must retain native npm publication");
  // Cargo still verifies the exact validated source, without waiting for binaries.
  assert.match(cargo, /^ {4}needs:\n {6}- prepare\n {6}- validate$/m);
  assert.match(cargo, /ref: \$\{\{ needs\.prepare\.outputs\.release-sha \}\}/);
  assert.match(cargo, /persist-credentials: false/);
  assert.match(cargo, /cargo publish --locked -p "\$pkg"/);
  assert.doesNotMatch(cargo, /--no-verify|id-token: write|contents: write|RELEASE_TOKEN/);
  assert.match(cargo, /already exists on crates\.io index/);
  assert.match(cargo, /crates\.io\/api\/v1\/crates\/\$pkg\/\$version/);
  assert.doesNotMatch(npm, /Publish Cargo crates|Install Rust|CARGO_REGISTRY_TOKEN|publish-cargo/);
  assert.match(npm, /^ {4}needs:\n {6}- prepare\n {6}- validate\n {6}- build-native$/m);
});

test("ESLint publication overlaps native builds without gating root verification", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  const eslint = workflow.match(/^ {2}publish-eslint:[\s\S]*?(?=^ {2}[a-z])/m)?.[0];
  const root = workflow.match(/^ {2}publish-js:[\s\S]*?(?=^ {2}[a-z])/m)?.[0];
  const verifyRoot = workflow.match(/^ {2}verify-npm-root:[\s\S]*$/m)?.[0];
  assert.ok(eslint, "source-only ESLint publication must have its own job");
  assert.ok(root);
  assert.ok(verifyRoot);
  assert.match(eslint, /^ {4}needs:\n {6}- prepare\n {6}- validate$/m);
  assert.match(eslint, /ref: \$\{\{ needs\.prepare\.outputs\.release-sha \}\}/);
  assert.match(eslint, /^ {6}contents: read$/m);
  assert.match(eslint, /^ {6}id-token: write$/m);
  assert.match(eslint, /persist-credentials: false/);
  assert.doesNotMatch(eslint, /RELEASE_TOKEN|NPM_TOKEN/);
  assert.match(eslint, /^ {12}eslint-plugin-no-mistakes$/m);
  assert.doesNotMatch(eslint, /^ {12}no-mistakes$/m);
  assert.match(eslint, /npm publish "\.\/packages\/\$pkg" --provenance --access public/);
  assert.match(eslint, /--timeout-ms 1800000/);
  assert.doesNotMatch(root, /eslint-plugin-no-mistakes/);
  assert.doesNotMatch(verifyRoot, /publish-eslint/);
});

test("release does not publish no-mistakes until every platform tarball installs", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  assert.match(workflow, /wait-npm-tarball\.js/);
  assert.match(workflow, /smoke-published-native\.js/);
  assert.match(workflow, /metadata exists without a fetchable tarball; waiting/);
  assert.match(workflow, /--timeout-ms 0/);
  const verifyPlatform = workflow.match(/^ {2}verify-npm-platform:[\s\S]*?(?=^ {2}publish-js:)/m);
  const verifyRoot = workflow.match(/^ {2}verify-npm-root:[\s\S]*$/m);
  assert.ok(verifyPlatform, "release workflow must verify platform packages before JS publish");
  assert.ok(verifyRoot, "release workflow must verify the root package after JS publish");
  assert.match(verifyPlatform[0], /needs:\n {6}- prepare\n {6}- publish/);
  assert.match(verifyPlatform[0], /os: macos-15, package: no-mistakes-darwin-arm64/);
  assert.match(verifyPlatform[0], /os: ubuntu-22\.04, package: no-mistakes-linux-x64-gnu/);
  assert.match(verifyPlatform[0], /os: ubuntu-22\.04-arm, package: no-mistakes-linux-arm64-gnu/);
  assert.match(verifyPlatform[0], /os: windows-2025, package: no-mistakes-win32-x64-msvc/);
  assert.match(verifyPlatform[0], /persist-credentials: false/);
  assert.match(verifyRoot[0], /persist-credentials: false/);
  assert.match(verifyRoot[0], /needs:\n {6}- prepare\n {6}- publish-js/);
  assert.match(verifyRoot[0], /no-mistakes@\$version/);
});

test("native CI jobs run only platform-specific Rust tests", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
  const nativeJob = workflow.match(/^ {2}native-tests:[\s\S]*?(?=^ {2}[a-z])/m);
  assert.ok(nativeJob, "ci.yml must define native-tests");
  const body = nativeJob[0];

  // Bash `\` continuations are one argv to cargo; a newline-limited regex
  // would miss `--all-features` on the next physical line.
  const unfoldedNativeJob = body.replace(/\\\r?\n[ \t]*/g, " ");
  assert.doesNotMatch(
    unfoldedNativeJob,
    /cargo test\b[^\r\n]*--workspace\b/,
    "native jobs must not compile or run the Linux full suite",
  );
  assert.match(
    body,
    /cargo test --locked -p no-mistakes --test macos_invocation/,
    "macOS native jobs must run the focused invocation test target",
  );
  assert.doesNotMatch(body, /cargo test --locked -p no-mistakes --lib/);
  assert.doesNotMatch(
    unfoldedNativeJob,
    /cargo test\b[^\r\n]*--all-features\b/,
    "native jobs must not pass --all-features to cargo test, including across bash line continuations",
  );
  assert.match(
    body,
    /cargo test --locked -p no-mistakes --test windows_job_object/,
    "Windows must run the Job Object regression as an integration test",
  );
  const macosInvocation = readFileSync(
    join(repoRoot, "crates", "no-mistakes", "tests", "macos_invocation.rs"),
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
  assert.match(body, /real-napi-api\.test\.js/);
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
    /bash \.github\/scripts\/rust-validation\.sh test/,
    "Linux coverage keeps the shared full-suite validation command",
  );
});

// Keep native build/test debug settings aligned without relying on Cargo to
// reuse the preceding library build for integration tests.
test("native CI omits debug metadata without weakening runtime checks", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "ci.yml"), "utf8");
  const body = workflow.match(/^ {2}native-tests:[\s\S]*?(?=^ {2}[a-z])/m)?.[0];
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
  for (const name of ["Build native CLI and N-API addon", "Run platform-specific Rust tests"]) {
    const step = body.split(`- name: ${name}\n`)[1]?.split("\n      - name:")[0];
    assert.ok(step, `${name} must remain enabled`);
    assert.match(step, /^ {8}timeout-minutes: 10$/m);
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
  for (const command of [
    "cargo fmt --all -- --check",
    "cargo clippy --locked --workspace --all-targets --all-features -- -D warnings",
    'bash "$RUNNER_TEMP/validation-tools/.github/scripts/rust-validation.sh" build',
    'bash "$RUNNER_TEMP/validation-tools/.github/scripts/rust-validation.sh" test',
    'bash "$RUNNER_TEMP/validation-tools/.github/scripts/rust-validation.sh" doc',
    "cargo publish --dry-run --locked -p no-mistakes",
    "pnpm run build:js",
  ])
    assert.ok(rust.includes(`run: ${command}\n`), command);
  assert.match(rust, /shared-key: validate/);
  assert.match(rust, /postgresql-18/);
  for (const command of [
    "pnpm run build:js",
    "pnpm run fmt:js:check",
    "pnpm run lint:js",
    "pnpm run lint:spell",
    "pnpm test",
  ]) {
    assert.ok(js.includes(`run: ${command}\n`), command);
  }
  assert.match(js, /playwright install --with-deps chromium/);
  assert.match(js, /pnpm --filter "\$pkg" pack --dry-run/);
  assert.doesNotMatch(js, /cargo|Install Rust/);
  assert.match(gate, /needs:\n {6}- validate-rust\n {6}- validate-js/);
  assert.match(gate, /if: \$\{\{ always\(\) \}\}/);
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
  for (const publisher of ["publish", "publish-cargo", "publish-eslint"]) {
    assert.match(job(publisher), /^ {6}- validate$/m);
  }
});
