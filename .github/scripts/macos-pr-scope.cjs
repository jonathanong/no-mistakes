// Pull-request macOS jobs. Unknown paths run them. See docs/development/ci-builds.md.

const TRIGGER_PREFIXES = [
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
];

const INDEPENDENT_PREFIXES = [
  "docs/",
  "crates/no-mistakes/src/codebase/postgres/",
  "crates/no-mistakes/src/postgres",
  "crates/no-mistakes/src/codebase/rules/postgres_",
  "test-cases/rules/postgres-",
  "test-cases/postgres/",
  "test-cases/postgres-facts/",
  "fixtures/postgres/",
  "fixtures/postgres-facts/",
  "fixtures/check/postgres-",
  "fixtures/config/postgres-",
  "fixtures/node-api/postgres-",
  "fixtures/performance/postgres-",
  "fixtures/rules/postgres-",
  "crates/no-mistakes/tests/common/snapshots/postgres_",
  "crates/no-mistakes/tests/cli_postgres_",
  "crates/no-mistakes/tests/cli_check_rules_postgres",
  "crates/no-mistakes/tests/cli_check_postgres_",
  "crates/no-mistakes/tests/executed_postgres_",
  "crates/no-mistakes/tests/postgres_",
  "packages/eslint-plugin-no-mistakes/",
];

const MARKDOWN_CODE_ROOTS = ["crates/", "packages/", "fixtures/", "test-cases/", "tests/"];

function isCargoBuildScript(path) {
  return /(^|\/)build\.rs$/.test(path) && !path.includes("/src/");
}

function isManifest(path) {
  return (
    /(^|\/)Cargo\.toml$/.test(path) ||
    /(^|\/)Cargo\.lock$/.test(path) ||
    /(^|\/)rust-toolchain\.toml$/.test(path)
  );
}

function isIndependentMarkdown(path) {
  if (!path.endsWith(".md") && !path.endsWith(".mdx")) return false;
  if (path === "AGENTS.md" || path.endsWith("/AGENTS.md")) return true;
  return !MARKDOWN_CODE_ROOTS.some((prefix) => path.startsWith(prefix));
}

function triggersMacos(path) {
  if (isManifest(path) || isCargoBuildScript(path)) return true;
  if (TRIGGER_PREFIXES.some((prefix) => path.startsWith(prefix))) return true;
  if (isIndependentMarkdown(path)) return false;
  if (INDEPENDENT_PREFIXES.some((prefix) => path.startsWith(prefix))) return false;
  return true;
}

function macosRequired(changes) {
  return changes.length === 0 || changes.some(({ path }) => triggersMacos(path));
}

function readChangedFiles() {
  const fields = require("node:fs").readFileSync(0).toString("utf8").split("\0");
  if (fields.pop() !== "" || fields.length % 2 !== 0) {
    throw new Error("Expected NUL-terminated git diff name-status pairs");
  }
  const changes = [];
  for (let index = 0; index < fields.length; index += 2) {
    const status = fields[index];
    const path = fields[index + 1];
    if (!/^[ADMT]$/.test(status) || !path) {
      throw new Error("Expected added, deleted, modified, or type-changed git diff path");
    }
    changes.push({ status, path });
  }
  return changes;
}

if (require.main === module) {
  if (process.argv.includes("--all")) {
    process.stdout.write("macos=true\n");
    process.exit(0);
  }
  process.stdout.write(`macos=${macosRequired(readChangedFiles())}\n`);
}

module.exports = {
  INDEPENDENT_PREFIXES,
  TRIGGER_PREFIXES,
  macosRequired,
  triggersMacos,
};
