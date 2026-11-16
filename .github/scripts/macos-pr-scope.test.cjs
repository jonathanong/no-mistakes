const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const { workflowJob } = require("./workflow-contract.cjs");
const {
  INDEPENDENT_PREFIXES,
  TRIGGER_PREFIXES,
  macosRequired,
  triggersMacos,
} = require("./macos-pr-scope.cjs");

const root = path.resolve(__dirname, "../..");
const source = readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
const selector = path.join(__dirname, "macos-pr-scope.cjs");
const modified = (paths) => paths.map((file) => ({ status: "M", path: file }));

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

function nameStatus(repo, from, to) {
  const result = spawnSync(
    "git",
    ["-C", repo, "diff", "--no-renames", "--name-status", "-z", from, to],
    { encoding: "utf8" },
  );
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}

test("enumerated paths run macOS and platform-independent paths do not", () => {
  for (const prefix of [
    ".github/workflows/",
    ".github/actions/",
    ".github/scripts/",
    ".github/actionlint.yaml",
    ".cargo/",
    "packages/no-mistakes",
    "crates/no-mistakes/src/napi_api/",
    "crates/no-mistakes/src/check_tasks/filesystem",
    "crates/no-mistakes/src/playwright/fsutil.rs",
    "crates/no-mistakes/src/check_discovery",
    "crates/no-mistakes/src/invocation/",
    "crates/no-mistakes/src/diagnostics/memory.rs",
    "crates/no-mistakes/src/diagnostics/resources/",
    "crates/no-mistakes/src/codebase/analysis_session/visible/",
    "crates/no-mistakes/src/codebase/ts_source/discovery/",
    "crates/no-mistakes/tests/macos_invocation.rs",
    "crates/no-mistakes/tests/cli_invocation/",
    "crates/no-mistakes/src/codebase/rules/filesystem_dispatch",
    "crates/no-mistakes/src/codebase/rules/path_filter",
    "crates/no-mistakes/src/codebase/rules/banned_paths",
    "crates/no-mistakes/src/codebase/rules/banned_renamed_files",
    "crates/no-mistakes/src/codebase/rules/config_path_references",
    "crates/no-mistakes/src/codebase/rules/file_extension_policy",
    "crates/no-mistakes/src/codebase/rules/file_matching",
    "crates/no-mistakes/src/codebase/rules/test_no_live_repository_scan",
    "crates/no-mistakes/src/codebase/rules/vitest_ci_path_coverage",
    "crates/no-mistakes/src/codebase/rules/tsconfig_file_coverage",
    "crates/no-mistakes/src/codebase/rules/require_files_in_subdirs",
    "crates/no-mistakes/src/codebase/rules/selected_files",
    "crates/no-mistakes/src/codebase/rules/suppression_absolute_paths_tests.rs",
    "crates/no-mistakes/src/codebase/rules/postgres_sql_cross_file",
    "crates/no-mistakes/src/postgres_catalog",
  ])
    assert.ok(TRIGGER_PREFIXES.includes(prefix), prefix);
  for (const prefix of [
    "docs/",
    "crates/no-mistakes/src/codebase/postgres/",
    "crates/no-mistakes/src/codebase/rules/postgres_",
    "test-cases/rules/postgres-",
    "fixtures/postgres-facts/",
    "packages/eslint-plugin-no-mistakes/",
  ])
    assert.ok(INDEPENDENT_PREFIXES.includes(prefix), prefix);

  for (const file of [
    "Cargo.toml",
    "Cargo.lock",
    "crates/no-mistakes/Cargo.toml",
    "rust-toolchain.toml",
    ".cargo/config.toml",
    "crates/no-mistakes/build.rs",
    ".github/workflows/ci.yml",
    ".github/workflows/release.yml",
    ".github/scripts/macos-pr-scope.cjs",
    ".github/scripts/macos-pr-scope.test.cjs",
    ".github/scripts/build-native.cjs",
    "packages/no-mistakes/package.json",
    "packages/no-mistakes/scripts/local-stage-native.js",
    "packages/no-mistakes-darwin-arm64/package.json",
    "packages/no-mistakes-linux-x64-gnu/package.json",
    "packages/no-mistakes-win32-x64-msvc/package.json",
    "crates/no-mistakes/src/napi_api/filesystem.rs",
    "crates/no-mistakes/src/napi_api/options.rs",
    "crates/no-mistakes/src/codebase/rules/filesystem_dispatch.rs",
    "crates/no-mistakes/src/codebase/rules/filesystem_dispatch/run_rule_engines/postgres/schema.rs",
    "crates/no-mistakes/src/codebase/rules/path_filter.rs",
    "crates/no-mistakes/src/codebase/rules/banned_paths.rs",
    "crates/no-mistakes/src/codebase/rules/config_path_references/references.rs",
    "crates/no-mistakes/src/codebase/rules/test_no_live_repository_scan.rs",
    "crates/no-mistakes/src/codebase/rules/postgres_sql_cross_file.rs",
    "crates/no-mistakes/src/check_tasks/filesystem.rs",
    "crates/no-mistakes/src/playwright/fsutil.rs",
    "crates/no-mistakes/src/check_discovery/tests/path_entries.rs",
    "crates/no-mistakes/src/invocation/child.rs",
    "crates/no-mistakes/src/diagnostics/memory.rs",
    "crates/no-mistakes/tests/macos_invocation.rs",
    "crates/no-mistakes/tests/cli_invocation/fsmonitor.rs",
    "crates/no-mistakes/src/lib.rs",
    "crates/no-mistakes/src/codebase/rules/agents_md_max_size.rs",
    "crates/no-mistakes/src/postgres_catalog.rs",
    "crates/no-mistakes/src/postgres_catalog/connection.rs",
  ])
    assert.equal(triggersMacos(file), true, file);
  assert.ok(
    "crates/no-mistakes/src/postgres_catalog.rs".startsWith("crates/no-mistakes/src/postgres"),
  );
  assert.ok(
    "crates/no-mistakes/src/postgres_catalog/connection.rs".startsWith(
      "crates/no-mistakes/src/postgres",
    ),
  );

  for (const file of [
    "crates/no-mistakes/src/codebase/rules/postgres_bounded_statements.rs",
    "crates/no-mistakes/src/codebase/rules/postgres_bounded_statements/query.rs",
    "crates/no-mistakes/src/codebase/postgres/catalog/build.rs",
    "crates/no-mistakes/src/postgres.rs",
    "crates/no-mistakes/src/postgres/tests.rs",
    "test-cases/rules/postgres-bounded-statements/issue-3389/fixture/queries.sql",
    "fixtures/postgres-facts/embedded/variants-single-legacy.txt",
    "docs/rules/postgres-bounded-statements.md",
    "docs/postgres-facts/row-bound-facts.md",
    "docs/development/ci-builds.md",
    "CONTRIBUTING.md",
    "README.md",
    "crates/AGENTS.md",
    "packages/eslint-plugin-no-mistakes/index.js",
    "crates/no-mistakes/tests/common/snapshots/postgres_bounded_statements_issue_3389.snap",
    "crates/no-mistakes/tests/cli_postgres_snapshot_bounded_chains.rs",
    "crates/no-mistakes/tests/cli_check_rules_postgres.rs",
  ])
    assert.equal(triggersMacos(file), false, file);
});

test("a postgres-rule-only diff skips macOS and any other path runs it", () => {
  const postgres = modified([
    "crates/no-mistakes/src/codebase/rules/postgres_bounded_statements.rs",
    "crates/no-mistakes/src/codebase/postgres/catalog/build.rs",
    "test-cases/rules/postgres-bounded-statements/issue-3389/fixture/queries.sql",
    "docs/rules/postgres-bounded-statements.md",
  ]);
  assert.equal(macosRequired(postgres), false);
  assert.equal(macosRequired([]), true);
  assert.equal(
    macosRequired(
      modified([...postgres.map(({ path: file }) => file), ".github/workflows/ci.yml"]),
    ),
    true,
  );
  assert.equal(macosRequired(modified(["crates/no-mistakes/src/lib.rs"])), true);
});

test("CLI reads NUL-separated name-status pairs and fails closed without a base", () => {
  const postgres =
    "M\0crates/no-mistakes/src/codebase/rules/postgres_bounded_statements.rs\0" +
    "M\0docs/rules/postgres-bounded-statements.md\0";
  assert.equal(run(postgres).stdout, "macos=false\n");
  assert.equal(run(`${postgres}M\0Cargo.lock\0`).stdout, "macos=true\n");
  assert.equal(run("", ["--all"]).stdout, "macos=true\n");
  assert.equal(run("").stdout, "macos=true\n");
  assert.notEqual(run("R100\0a.rs\0").status, 0);
});

test("pull requests gate macOS families and aggregates accept only that skip", () => {
  assert.doesNotMatch(source, /every PR runs all test families/);
  assert.match(
    source,
    /macos: \$\{\{ github\.event_name != 'pull_request' \|\| steps\.macos-paths\.outputs\.macos == 'true' \}\}/,
  );
  assert.match(source, /node --test[^\n]*macos-pr-scope\.test\.cjs/);
  assert.match(source, /docs-only: \$\{\{ github\.event_name == 'pull_request' &&/);
  assert.doesNotMatch(source, /needs\.changes\.outputs\.docs-only/);
  const release = readFileSync(path.join(root, ".github/workflows/release.yml"), "utf8");
  assert.doesNotMatch(release, /macos-pr-scope|outputs\.macos/);

  for (const name of [
    "native-build-macos",
    "native-test-build-macos",
    "native-rust-tests-macos",
    "native-api-tests-macos",
  ]) {
    assert.match(
      workflowJob(source, name),
      /if: \$\{\{ needs\.changes\.outputs\.macos == 'true' \}\}/,
    );
  }
  for (const name of ["native-build", "native-test-build", "native-rust-tests", "native-api-tests"])
    assert.doesNotMatch(workflowJob(source, name), /needs\.changes\.outputs\.macos/);
  assert.doesNotMatch(workflowJob(source, "native-compile-profile"), /outputs\.macos/);
  assert.doesNotMatch(workflowJob(source, "native-compile-profile"), /ci-macos-main/);
  assert.match(workflowJob(source, "native-compile-profile"), /cancel-in-progress: false/);
  assert.doesNotMatch(workflowJob(source, "tests-gate"), /MACOS_REQUIRED/);
  assert.match(
    source,
    /group: \$\{\{ github\.workflow \}\}-\$\{\{ github\.event_name == 'pull_request' && format\('pr-\{0\}', github\.event\.pull_request\.number\) \|\| github\.sha \}\}/,
  );

  const mainPush = "github.event_name == 'push' && github.ref == 'refs/heads/main'";
  for (const name of ["native-build-macos", "native-test-build-macos"]) {
    const body = workflowJob(source, name);
    assert.match(
      body,
      new RegExp(
        `group: \\$\\{\\{ ${mainPush} && format\\('ci-macos-main-\\{0\\}-all', github\\.job\\) \\|\\| format\\('ci-macos-\\{0\\}-\\{1\\}-all', github\\.run_id, github\\.job\\) \\}\\}`,
      ),
      name,
    );
    assert.match(body, /cancel-in-progress: false/, name);
  }
  for (const name of ["native-rust-tests-macos", "native-api-tests-macos"]) {
    const body = workflowJob(source, name);
    assert.match(
      body,
      new RegExp(
        `group: \\$\\{\\{ ${mainPush} && format\\('ci-macos-main-\\{0\\}-\\{1\\}', github\\.job, matrix\\.shard\\) \\|\\| format\\('ci-macos-\\{0\\}-\\{1\\}-\\{2\\}', github\\.run_id, github\\.job, matrix\\.shard\\) \\}\\}`,
      ),
      name,
    );
    assert.match(body, /cancel-in-progress: false/, name);
  }
  for (const name of ["native-build", "native-test-build", "native-rust-tests", "native-api-tests"])
    assert.doesNotMatch(workflowJob(source, name), /concurrency:|ci-macos-main/);
  for (const name of ["retain-native-build-macos-cache", "retain-native-test-build-macos-cache"])
    assert.match(workflowJob(source, name), /needs\.[^.]+\.result == 'success'/);

  const native = workflowJob(source, "native-tests");
  assert.match(native, /MACOS_REQUIRED: \$\{\{ needs\.changes\.outputs\.macos \}\}/);
  assert.match(native, /MACOS_EVENT: \$\{\{ github\.event_name \}\}/);
  assert.match(native, /MACOS_REF: \$\{\{ github\.ref \}\}/);
  const needs = native
    .match(/needs:\s*\[([^\]]+)\]/)[1]
    .split(",")
    .map((name) => name.trim());
  assert.ok(needs.includes("changes"));
  const script = native.split("        run: |\n")[1].replace(/^          /gm, "");
  const macosJobs = needs.filter((name) => name.endsWith("-macos"));
  assert.deepEqual(macosJobs.sort(), [
    "native-api-tests-macos",
    "native-build-macos",
    "native-rust-tests-macos",
    "native-test-build-macos",
  ]);
  const runGate = (results, required, event = "", ref = "") =>
    spawnSync("bash", ["-eu", "-c", script], {
      encoding: "utf8",
      env: {
        ...process.env,
        MACOS_REQUIRED: required,
        MACOS_EVENT: event,
        MACOS_REF: ref,
        RESULTS: JSON.stringify(results),
      },
    });
  const success = Object.fromEntries(needs.map((name) => [name, { result: "success" }]));
  const skippedMacos = {
    ...success,
    ...Object.fromEntries(macosJobs.map((name) => [name, { result: "skipped" }])),
  };
  assert.equal(runGate(skippedMacos, "false").status, 0);
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
  const cancelledMacos = {
    ...success,
    ...Object.fromEntries(macosJobs.map((name) => [name, { result: "cancelled" }])),
  };
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
});

test("pull requests compare macOS paths from the merge base", () => {
  const step = source.slice(
    source.indexOf("      - name: Check macOS paths"),
    source.indexOf("  quality-source:"),
  );
  assert.match(step, /\[\[ "\$GITHUB_EVENT_NAME" == "pull_request" \]\]/);
  assert.match(step, /git merge-base "\$BASE_SHA" "\$HEAD_SHA"/);
  assert.match(step, /git fetch --no-tags --deepen=300 origin "\$BASE_SHA" "\$HEAD_SHA"/);
  assert.match(
    step,
    /macos-pr-scope\.cjs --diff-base "\$GITHUB_EVENT_NAME" "\$BASE_SHA" "\$HEAD_SHA"/,
  );
  assert.match(step, /git diff --no-renames --name-status -z "\$DIFF_BASE" "\$HEAD_SHA"/);
  assert.equal(step.split("macos-pr-scope.cjs --all").length - 1, 4);
  const docs = source.slice(
    source.indexOf("      - name: Check docs-only paths"),
    source.indexOf("      - name: Check macOS paths"),
  );
  assert.match(docs, /git diff --no-renames --name-status -z "\$BASE_SHA" "\$HEAD_SHA"/);
  assert.doesNotMatch(docs, /merge-base|--diff-base/);

  const repo = mkdtempSync(path.join(tmpdir(), "macos-pr-scope-"));
  const postgres = "crates/no-mistakes/src/codebase/rules/postgres_bounded_statements.rs";
  try {
    git(repo, ["init", "-q", "-b", "main"]);
    mkdirSync(path.join(repo, "crates/no-mistakes/src/codebase/rules"), { recursive: true });
    mkdirSync(path.join(repo, ".github/workflows"), { recursive: true });
    writeFileSync(path.join(repo, "README.md"), "base\n");
    git(repo, ["add", "README.md"]);
    git(repo, ["commit", "-qm", "base"]);
    git(repo, ["checkout", "-q", "-b", "pr"]);
    writeFileSync(path.join(repo, postgres), "rule\n");
    git(repo, ["add", "-A"]);
    git(repo, ["commit", "-qm", "pr"]);
    const head = git(repo, ["rev-parse", "HEAD"]);
    git(repo, ["checkout", "-q", "main"]);
    writeFileSync(path.join(repo, ".github/workflows/ci.yml"), "workflow\n");
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

    const pullRequestDiff = nameStatus(repo, mergeBase, head);
    assert.equal(run(pullRequestDiff).stdout, "macos=false\n");
    assert.match(pullRequestDiff, /postgres_bounded_statements\.rs/);
    assert.doesNotMatch(pullRequestDiff, /workflows\/ci\.yml/);
    const endpointDiff = nameStatus(repo, base, head);
    assert.match(endpointDiff, /workflows\/ci\.yml/);
    assert.equal(run(endpointDiff).stdout, "macos=true\n");

    git(repo, ["checkout", "-q", "pr"]);
    mkdirSync(path.join(repo, ".github/workflows"), { recursive: true });
    writeFileSync(path.join(repo, ".github/workflows/release.yml"), "release\n");
    git(repo, ["add", "-A"]);
    git(repo, ["commit", "-qm", "pr-workflow"]);
    const workflowHead = git(repo, ["rev-parse", "HEAD"]);
    const workflowBase = run("", ["--diff-base", "pull_request", base, workflowHead], repo);
    assert.equal(workflowBase.status, 0, workflowBase.stderr);
    assert.equal(run(nameStatus(repo, workflowBase.stdout.trim(), workflowHead)).stdout, "macos=true\n");

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
