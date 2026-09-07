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
    ["darwin", "x64", "x86_64-apple-darwin"],
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

test("release native build jobs enforce separate CLI and N-API execution bounds", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  const timeouts = new Map();
  for (const name of ["build-cli", "build-napi"]) {
    const job = workflow.match(
      new RegExp(`^ {2}${name}:[\\s\\S]*?(?=^ {2}(?:build-|publish:))`, "m"),
    );
    assert.ok(job, `release workflow must define ${name}`);
    const jobTimeout = job[0].match(/^ {4}timeout-minutes: (\d+)$/m);
    const stepTimeouts = [...job[0].matchAll(/^ {8}timeout-minutes: (\d+)$/gm)];
    const buildCliTimeout = job[0].match(/^ {6}- name: Build CLI\n {8}timeout-minutes: (\d+)$/m);
    assert.ok(jobTimeout, `${name} must define a timeout`);
    timeouts.set(name, {
      buildCli: buildCliTimeout ? Number(buildCliTimeout[1]) : undefined,
      job: Number(jobTimeout[1]),
      steps: stepTimeouts.map((timeout) => Number(timeout[1])),
    });
  }

  const cli = timeouts.get("build-cli");
  assert.equal(cli.job, 80, "CLI builds need an 80-minute job envelope");
  assert.equal(cli.buildCli, 45, "Build CLI needs a 45-minute cold-build budget");
  assert.ok(
    cli.steps.every((timeout) => timeout <= 45),
    "CLI steps must be at most 45 minutes",
  );

  const napi = timeouts.get("build-napi");
  assert.ok(napi.job <= 30, "N-API builds must be at most 30 minutes");
  assert.ok(
    napi.steps.every((timeout) => timeout <= 25),
    "N-API steps must be at most 25 minutes",
  );
});

test("release syncs optional native package versions and publishes only through npm OIDC", () => {
  const workflow = readFileSync(join(repoRoot, ".github", "workflows", "release.yml"), "utf8");
  assert.match(workflow, /sync-native-package-versions\.js "\$version"/);
  assert.match(workflow, /npm publish "\.\/packages\/\$pkg" --provenance --access public/);
  assert.doesNotMatch(workflow, /NPM_TOKEN|pnpm[^\n]* publish/);
  for (const name of [
    "no-mistakes-darwin-arm64",
    "no-mistakes-darwin-x64",
    "no-mistakes-linux-arm64-gnu",
    "no-mistakes-linux-x64-gnu",
    "no-mistakes-win32-x64-msvc",
  ]) {
    const platformIndex = workflow.indexOf(`            ${name}`);
    const mainIndex = workflow.indexOf("            no-mistakes \\\n", platformIndex);
    assert.ok(
      platformIndex >= 0 && mainIndex > platformIndex,
      `${name} must publish before no-mistakes`,
    );
  }
  assert.match(workflow, /Expected exactly one N-API addon candidate/);
  assert.match(workflow, /expected_magic='Mach-O\.\*arm64'/);
  assert.match(workflow, /expected_magic='ELF 64-bit\.\*x86-64'/);
  assert.match(workflow, /expected_magic='PE32\\\+\.\*x86-64'/);
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
