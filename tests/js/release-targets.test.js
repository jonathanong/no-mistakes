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
  assert.match(body, /^ {4}timeout-minutes: 80$/m, "native builds keep an 80-minute job envelope");
  assert.match(
    body,
    /^ {6}- name: Build CLI and N-API addon\n {8}timeout-minutes: 45$/m,
    "combined compile keeps a 45-minute cold-build budget",
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
  assert.match(publishJs[0], /            no-mistakes \\/);
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
    /cargo test --locked -p no-mistakes --lib/,
    "macOS native jobs must compile only the no-mistakes lib tests",
  );
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
  assert.match(body, /rust_test: ["']invocation::["']/);
  assert.match(
    body,
    /head\.repo\.full_name == github\.repository/,
    "native timing comments must not run on fork PRs where GITHUB_TOKEN cannot write",
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
    /cargo test --workspace --all-features/,
    "Linux coverage keeps the full-suite spelling the native guard must reject",
  );
});
