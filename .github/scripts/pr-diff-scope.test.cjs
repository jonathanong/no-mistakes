const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");

const root = path.resolve(__dirname, "../..");
const source = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
const selector = path.join(__dirname, "pr-diff-scope.cjs");

function run(input, args = [], cwd) {
  return spawnSync(process.execPath, [selector, ...args], { input, encoding: "utf8", cwd });
}

function git(repo, args) {
  const result = spawnSync(
    "git",
    [
      "-C",
      repo,
      "-c",
      "commit.gpgsign=false",
      "-c",
      "user.email=test@example.com",
      "-c",
      "user.name=test",
      ...args,
    ],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 0, `${args.join(" ")}\n${result.stderr}`);
  return result.stdout.trim();
}

test("CLI requires a diff-base request and rejects unknown input", () => {
  assert.notEqual(run("").status, 0);
  assert.notEqual(run("R100\0a.rs\0", ["--all"]).status, 0);
});

test("pull requests run one macOS smoke job and the full macOS suite runs elsewhere", () => {
  assert.match(source, /macos: \$\{\{ github\.event_name != 'pull_request' \}\}/);
  assert.match(
    source,
    /macos-smoke: \$\{\{ github\.event_name == 'pull_request' && steps\.rust-paths\.outputs\.rust != 'false' \}\}/,
  );
  assert.match(source, /node --test[^\n]*pr-diff-scope\.test\.cjs/);
  assert.match(source, /docs-only: \$\{\{ github\.event_name == 'pull_request' &&/);
  assert.doesNotMatch(source, /needs\.changes\.outputs\.docs-only/);
  const release = readFileSync(path.join(root, ".github/workflows/release.yml"), "utf8");
  assert.doesNotMatch(release, /pr-diff-scope|outputs\.macos/);

  for (const name of [
    "native-build-macos",
    "native-test-build-macos",
    "native-rust-tests-macos",
    "native-api-tests-macos",
  ]) {
    assert.match(
      workflowJob(source, name),
      /if: \$\{\{ needs\.changes\.outputs\.macos == 'true'(?: && needs\.changes\.outputs\.rust == 'true')? \}\}/,
    );
  }
  // Windows jobs are not queue-bound and run on every pull request unchanged.
  for (const name of ["native-build", "native-test-build", "native-rust-tests", "native-api-tests"])
    assert.doesNotMatch(workflowJob(source, name), /needs\.changes\.outputs\.macos/);
  assert.doesNotMatch(workflowJob(source, "native-compile-profile"), /outputs\.macos/);
  assert.doesNotMatch(workflowJob(source, "native-compile-profile"), /ci-macos-main/);
  assert.match(workflowJob(source, "native-compile-profile"), /cancel-in-progress: false/);
  assert.doesNotMatch(workflowJob(source, "tests-gate"), /MACOS_REQUIRED/);
  assert.match(
    source,
    /group: \$\{\{ github\.workflow \}\}-\$\{\{ github\.event_name == 'pull_request' && format\('pr-\{0\}', github\.event\.pull_request\.number\) \|\| github\.ref \}\}\n {2}cancel-in-progress: \$\{\{ github\.event_name == 'pull_request' \}\}/,
  );

  const mainPush = "github.event_name == 'push' && github.ref == 'refs/heads/main'";
  for (const [name, slot] of [
    ["native-build-macos", "native-build-macos-all"],
    ["native-test-build-macos", "native-test-build-macos-all"],
  ]) {
    const body = workflowJob(source, name);
    assert.match(
      body,
      new RegExp(
        `group: \\$\\{\\{ ${mainPush} && 'ci-macos-main-${slot}' \\|\\| format\\('ci-macos-\\{0\\}-${slot}', github\\.run_id\\) \\}\\}`,
      ),
      name,
    );
    assert.doesNotMatch(body, /github\.job/);
    assert.match(body, /cancel-in-progress: false/, name);
  }
  for (const name of ["native-rust-tests-macos", "native-api-tests-macos"]) {
    const body = workflowJob(source, name);
    assert.match(
      body,
      new RegExp(
        `group: \\$\\{\\{ ${mainPush} && format\\('ci-macos-main-${name}-\\{0\\}', matrix\\.shard\\) \\|\\| format\\('ci-macos-\\{0\\}-${name}-\\{1\\}', github\\.run_id, matrix\\.shard\\) \\}\\}`,
      ),
      name,
    );
    assert.doesNotMatch(body, /github\.job/);
    assert.match(body, /cancel-in-progress: false/, name);
  }
  for (const name of ["native-build", "native-test-build", "native-rust-tests", "native-api-tests"])
    assert.doesNotMatch(workflowJob(source, name), /concurrency:|ci-macos-main/);
  assert.doesNotMatch(workflowJob(source, "macos-smoke"), /concurrency:|ci-macos-main/);
  for (const name of ["retain-native-build-macos-cache", "retain-native-test-build-macos-cache"])
    assert.match(workflowJob(source, name), /needs\.[^.]+\.result == 'success'/);
});

test("the native aggregate accepts a skipped full macOS tier only on pull requests", () => {
  const native = workflowJob(source, "native-tests");
  assert.match(native, /MACOS_REQUIRED: \$\{\{ needs\.changes\.outputs\.macos \}\}/);
  assert.match(native, /MACOS_EVENT: \$\{\{ github\.event_name \}\}/);
  assert.match(native, /MACOS_REF: \$\{\{ github\.ref \}\}/);
  const needs = native
    .match(/needs:\s*\[([^\]]+)\]/)[1]
    .split(",")
    .map((name) => name.trim());
  assert.ok(needs.includes("changes"));
  assert.ok(needs.includes("macos-smoke"));
  const script = native.split("        run: |\n")[1].replace(/^          /gm, "");
  const macosJobs = needs.filter((name) => name.endsWith("-macos"));
  assert.deepEqual(macosJobs.sort(), [
    "native-api-tests-macos",
    "native-build-macos",
    "native-rust-tests-macos",
    "native-test-build-macos",
  ]);
  const runGate = (results, required, event = "", ref = "", rust = "true") =>
    spawnSync("bash", ["-eu", "-c", script], {
      encoding: "utf8",
      env: {
        ...process.env,
        MACOS_REQUIRED: required,
        MACOS_EVENT: event,
        MACOS_REF: ref,
        RUST_REQUIRED: rust,
        RESULTS: JSON.stringify(results),
      },
    });
  const success = Object.fromEntries(needs.map((name) => [name, { result: "success" }]));
  const withResult = (names, result) => ({
    ...success,
    ...Object.fromEntries(names.map((name) => [name, { result }])),
  });
  const skippedMacos = withResult(macosJobs, "skipped");

  // Pull request: the full macOS tier is skipped and the smoke job must pass.
  assert.equal(runGate(skippedMacos, "false", "pull_request").status, 0);
  assert.equal(
    runGate({ ...skippedMacos, "macos-smoke": { result: "failure" } }, "false").status,
    1,
  );
  assert.equal(
    runGate({ ...skippedMacos, "macos-smoke": { result: "skipped" } }, "false").status,
    1,
  );
  assert.equal(
    runGate({ ...skippedMacos, "macos-smoke": { result: "skipped" } }, "false", "", "", "false")
      .status,
    0,
  );
  // Full run: the smoke job is skipped and the full macOS jobs must pass.
  assert.equal(
    runGate(withResult(["macos-smoke"], "skipped"), "true", "push", "refs/heads/main").status,
    0,
  );
  assert.equal(runGate(skippedMacos, "true").status, 1);
  assert.equal(runGate(skippedMacos, "").status, 1);
  assert.equal(
    runGate({ ...skippedMacos, "native-build": { result: "skipped" } }, "false").status,
    1,
  );
  assert.equal(
    runGate({ ...skippedMacos, "native-build-macos": { result: "failure" } }, "false").status,
    1,
  );
  assert.equal(runGate({ ...success, changes: { result: "skipped" } }, "false").status, 1);
  assert.equal(
    runGate({ ...skippedMacos, "native-api-tests-macos": { result: "cancelled" } }, "false").status,
    1,
  );
  const cancelledMacos = withResult(macosJobs, "cancelled");
  assert.equal(runGate(cancelledMacos, "true", "push", "refs/heads/main").status, 0);
  assert.equal(runGate(cancelledMacos, "false", "push", "refs/heads/main").status, 0);
  assert.equal(runGate(cancelledMacos, "true", "pull_request", "refs/heads/main").status, 1);
  assert.equal(runGate(cancelledMacos, "true", "push", "refs/heads/ci/cold-build").status, 1);
  assert.equal(runGate(cancelledMacos, "true", "workflow_dispatch", "refs/heads/main").status, 1);
  assert.equal(
    runGate(
      { ...success, "native-build": { result: "cancelled" } },
      "true",
      "push",
      "refs/heads/main",
    ).status,
    1,
  );
  assert.equal(
    runGate(
      { ...success, "native-build-macos": { result: "failure" } },
      "true",
      "push",
      "refs/heads/main",
    ).status,
    1,
  );
  const supersededApi = {
    ...success,
    "native-build-macos": { result: "cancelled" },
    "native-api-tests-macos": { result: "skipped" },
  };
  const supersededRust = {
    ...success,
    "native-test-build-macos": { result: "cancelled" },
    "native-rust-tests-macos": { result: "skipped" },
  };
  assert.equal(runGate(supersededApi, "true", "push", "refs/heads/main").status, 0);
  assert.equal(runGate(supersededRust, "true", "push", "refs/heads/main").status, 0);
  assert.equal(runGate(supersededApi, "true", "pull_request", "refs/heads/main").status, 1);
  assert.equal(
    runGate(
      { ...success, "native-api-tests-macos": { result: "skipped" } },
      "true",
      "push",
      "refs/heads/main",
    ).status,
    1,
  );
});

test("pull requests compare Rust paths from the merge base", () => {
  const step = source.slice(
    source.indexOf("      - name: Check Rust paths"),
    source.indexOf("  quality-source:"),
  );
  assert.match(step, /\[\[ "\$GITHUB_EVENT_NAME" == "pull_request" \]\]/);
  assert.match(step, /git merge-base "\$BASE_SHA" "\$HEAD_SHA"/);
  assert.match(step, /git fetch --no-tags --deepen=300 origin "\$BASE_SHA" "\$HEAD_SHA"/);
  assert.match(
    step,
    /pr-diff-scope\.cjs --diff-base "\$GITHUB_EVENT_NAME" "\$BASE_SHA" "\$HEAD_SHA"/,
  );
  assert.match(step, /git diff --no-renames --name-status -z "\$DIFF_BASE" "\$HEAD_SHA"/);
  assert.equal(step.split("rust-pr-scope.cjs --all").length - 1, 4);
  const docs = source.slice(
    source.indexOf("      - name: Check docs-only paths"),
    source.indexOf("      - name: Check Rust paths"),
  );
  assert.match(docs, /git diff --no-renames --name-status -z "\$BASE_SHA" "\$HEAD_SHA"/);
  assert.doesNotMatch(docs, /merge-base|--diff-base/);

  const repo = mkdtempSync(path.join(tmpdir(), "pr-diff-scope-"));
  try {
    git(repo, ["init", "-q", "-b", "main"]);
    mkdirSync(path.join(repo, "src"), { recursive: true });
    writeFileSync(path.join(repo, "README.md"), "base\n");
    git(repo, ["add", "README.md"]);
    git(repo, ["commit", "-qm", "base"]);
    git(repo, ["checkout", "-q", "-b", "pr"]);
    writeFileSync(path.join(repo, "src/lib.rs"), "pr\n");
    git(repo, ["add", "-A"]);
    git(repo, ["commit", "-qm", "pr"]);
    const head = git(repo, ["rev-parse", "HEAD"]);
    git(repo, ["checkout", "-q", "main"]);
    writeFileSync(path.join(repo, "main.txt"), "main\n");
    git(repo, ["add", "-A"]);
    git(repo, ["commit", "-qm", "main"]);
    const base = git(repo, ["rev-parse", "HEAD"]);
    const mergeBase = git(repo, ["merge-base", base, head]);

    const selected = run("", ["--diff-base", "pull_request", base, head], repo);
    assert.equal(selected.status, 0, selected.stderr);
    assert.equal(selected.stdout.trim(), mergeBase);
    assert.notEqual(mergeBase, base);
    assert.equal(run("", ["--diff-base", "push", base, head], repo).stdout.trim(), base);
    assert.equal(
      run("", ["--diff-base", "workflow_dispatch", base, head], repo).stdout.trim(),
      base,
    );

    git(repo, ["checkout", "-q", "--orphan", "unrelated"]);
    writeFileSync(path.join(repo, "README.md"), "unrelated\n");
    git(repo, ["add", "README.md"]);
    git(repo, ["commit", "-qm", "unrelated"]);
    const unrelated = git(repo, ["rev-parse", "HEAD"]);
    assert.notEqual(run("", ["--diff-base", "pull_request", unrelated, head], repo).status, 0);
    assert.notEqual(run("", ["--diff-base", "push", "abc", head], repo).status, 0);
    assert.notEqual(run("", ["--diff-base", "pull_request", base], repo).status, 0);
  } finally {
    rmSync(repo, { recursive: true, force: true });
  }
});

test("pull requests run one Windows smoke job and the full Windows suite runs elsewhere", () => {
  assert.match(source, /windows: \$\{\{ github\.event_name != 'pull_request' \}\}/);
  assert.match(
    source,
    /windows-smoke: \$\{\{ github\.event_name == 'pull_request' && steps\.rust-paths\.outputs\.rust != 'false' \}\}/,
  );
  const fullWindows = [
    "native-build",
    "native-test-build",
    "native-rust-tests",
    "native-api-tests",
  ];
  for (const name of fullWindows) {
    const body = workflowJob(source, name);
    assert.match(body, /needs\.changes\.outputs\.windows == 'true'/, name);
    assert.match(body, /windows-2025/, name);
  }
  assert.doesNotMatch(workflowJob(source, "windows-smoke"), /concurrency:/);

  const native = workflowJob(source, "native-tests");
  assert.match(native, /WINDOWS_REQUIRED: \$\{\{ needs\.changes\.outputs\.windows \}\}/);
  const needs = native
    .match(/needs:\s*\[([^\]]+)\]/)[1]
    .split(",")
    .map((name) => name.trim());
  assert.ok(needs.includes("windows-smoke"));
  const script = native.split("        run: |\n")[1].replace(/^          /gm, "");
  const success = Object.fromEntries(needs.map((name) => [name, { result: "success" }]));
  const withResult = (names, result) => ({
    ...success,
    ...Object.fromEntries(names.map((name) => [name, { result }])),
  });
  const gate = (results, windows, rust = "true") =>
    spawnSync("bash", ["-eu", "-c", script], {
      encoding: "utf8",
      env: {
        ...process.env,
        MACOS_REQUIRED: "false",
        MACOS_EVENT: "pull_request",
        MACOS_REF: "refs/pull/1/merge",
        WINDOWS_REQUIRED: windows,
        RUST_REQUIRED: rust,
        RESULTS: JSON.stringify(results),
      },
    }).status;
  const skipped = withResult(fullWindows, "skipped");

  // Pull request: the full Windows jobs are skipped and the smoke job must pass.
  assert.equal(gate(skipped, "false"), 0);
  assert.equal(gate({ ...skipped, "windows-smoke": { result: "failure" } }, "false"), 1);
  assert.equal(gate({ ...skipped, "windows-smoke": { result: "skipped" } }, "false"), 1);
  assert.equal(gate({ ...skipped, "windows-smoke": { result: "skipped" } }, "false", "false"), 0);
  // Full run: the smoke job is skipped and the full Windows jobs must pass.
  assert.equal(gate(withResult(["windows-smoke"], "skipped"), "true"), 0);
  assert.equal(gate(withResult([...fullWindows, "windows-smoke"], "skipped"), "true"), 1);
  assert.equal(gate({ ...success, "native-api-tests": { result: "failure" } }, "true"), 1);
  assert.equal(gate({ ...skipped, "native-build": { result: "failure" } }, "false"), 1);
  assert.equal(gate(skipped, ""), 1);
});
