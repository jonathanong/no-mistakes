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

const macosJobs = ["native-api-tests-macos", "native-rust-tests-macos"];
const windowsJobs = ["native-test-build", "native-rust-tests", "native-api-tests"];

test("native suites use only the path scope to decide which jobs run", () => {
  for (const [platform, jobs] of [["macos", macosJobs], ["windows", windowsJobs]]) {
    for (const name of jobs) {
      const body = workflowJob(source, name);
      assert.equal(
        body.match(/^    if: (.+)$/m)?.[1],
        `\${{ needs.changes.outputs.${platform} == 'true' }}`,
        name,
      );
    }
  }
  assert.doesNotMatch(source, /^  (?:macos|windows)-smoke:/m);
  assert.match(source, /node --test[^\n]*pr-diff-scope\.test\.cjs/);
  assert.doesNotMatch(source, /needs\.changes\.outputs\.docs-only/);
  const release = readFileSync(path.join(root, ".github/workflows/release.yml"), "utf8");
  assert.doesNotMatch(release, /pr-diff-scope|outputs\.macos/);
});

test("main macOS jobs retain their own supersession slots", () => {
  const mainPush = "github.event_name == 'push' && github.ref == 'refs/heads/main'";
  for (const name of macosJobs) {
    const body = workflowJob(source, name);
    assert.ok(
      body.includes(
        `group: \${{ ${mainPush} && format('ci-macos-main-${name}-{0}', matrix.shard) || format('ci-macos-{0}-${name}-{1}', github.run_id, matrix.shard) }}`,
      ),
      name,
    );
    assert.doesNotMatch(body, /github\.job/);
    assert.match(body, /cancel-in-progress: false/);
  }
  for (const name of windowsJobs)
    assert.doesNotMatch(workflowJob(source, name), /concurrency:|ci-macos-main/);
  const profile = workflowJob(source, "native-compile-profile");
  assert.doesNotMatch(profile, /outputs\.macos|ci-macos-main/);
  assert.match(profile, /cancel-in-progress: false/);
});

function gate(jobs, result, required, event, ref = "refs/heads/main") {
  const results = { changes: { result: "success" } };
  for (const name of jobs) results[name] = { result };
  return spawnSync(process.execPath, [path.join(__dirname, "tests-gate.cjs")], {
    encoding: "utf8",
    env: {
      ...process.env,
      MACOS_REQUIRED: required,
      WINDOWS_REQUIRED: required,
      MACOS_EVENT: event,
      MACOS_REF: ref,
      RUST_REQUIRED: required,
      RESULTS: JSON.stringify(results),
    },
  }).status;
}

test("the aggregate accepts path skips and rejects suite failures on every event", () => {
  for (const event of ["pull_request", "push", "workflow_dispatch"]) {
    for (const jobs of [macosJobs, windowsJobs]) {
      assert.equal(gate(jobs, "skipped", "false", event), 0);
      assert.equal(gate(jobs, "skipped", "true", event), 1);
      assert.equal(gate(jobs, "skipped", "", event), 1);
      assert.equal(gate(jobs, "failure", "true", event), 1);
      assert.equal(gate(jobs, "failure", "false", event), 1);
    }
  }
});

test("only a superseded main push can tolerate cancellation of a macOS job", () => {
  assert.equal(gate(macosJobs, "cancelled", "true", "push"), 0);
  assert.equal(gate(windowsJobs, "cancelled", "true", "push"), 1);
  assert.equal(gate(macosJobs, "cancelled", "true", "push", "refs/heads/ci/cold-build"), 1);
  assert.equal(gate(macosJobs, "cancelled", "true", "pull_request"), 1);
  assert.equal(gate(macosJobs, "cancelled", "true", "workflow_dispatch"), 1);
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
  assert.match(step, /node \.github\/scripts\/rust-pr-scope\.cjs < "\$RUNNER_TEMP\/native-changes"/);
  assert.match(step, /node \.github\/scripts\/rust-pr-scope\.cjs --all/);
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
