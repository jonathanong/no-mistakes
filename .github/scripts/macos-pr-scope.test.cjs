const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { readFileSync } = require("node:fs");
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

function run(input, args = []) {
  return spawnSync(process.execPath, [selector, ...args], { input, encoding: "utf8" });
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
  ])
    assert.equal(triggersMacos(file), true, file);

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
      modified([
        ...postgres.map(({ path: file }) => file),
        ".github/workflows/ci.yml",
      ]),
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
  for (const name of [
    "native-build",
    "native-test-build",
    "native-rust-tests",
    "native-api-tests",
  ])
    assert.doesNotMatch(workflowJob(source, name), /needs\.changes\.outputs\.macos/);
  assert.doesNotMatch(workflowJob(source, "native-compile-profile"), /outputs\.macos/);
  assert.doesNotMatch(workflowJob(source, "tests-gate"), /MACOS_REQUIRED/);

  const native = workflowJob(source, "native-tests");
  assert.match(native, /MACOS_REQUIRED: \$\{\{ needs\.changes\.outputs\.macos \}\}/);
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
  const runGate = (results, required) =>
    spawnSync("bash", ["-eu", "-c", script], {
      encoding: "utf8",
      env: { ...process.env, MACOS_REQUIRED: required, RESULTS: JSON.stringify(results) },
    });
  const success = Object.fromEntries(needs.map((name) => [name, { result: "success" }]));
  const skippedMacos = {
    ...success,
    ...Object.fromEntries(macosJobs.map((name) => [name, { result: "skipped" }])),
  };
  assert.equal(runGate(skippedMacos, "false").status, 0);
  assert.equal(runGate(skippedMacos, "true").status, 1);
  assert.equal(runGate(skippedMacos, "").status, 1);
  assert.equal(runGate({ ...skippedMacos, "native-build": { result: "skipped" } }, "false").status, 1);
  assert.equal(runGate({ ...skippedMacos, "native-build-macos": { result: "failure" } }, "false").status, 1);
  assert.equal(runGate({ ...success, changes: { result: "skipped" } }, "false").status, 1);
  assert.equal(runGate({ ...skippedMacos, "native-api-tests-macos": { result: "cancelled" } }, "false").status, 1);
});
