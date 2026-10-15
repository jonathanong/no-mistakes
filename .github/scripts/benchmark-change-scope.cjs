const { lstatSync, readFileSync } = require("node:fs");
const { join } = require("node:path");

const ALL_SHARDS = "all";
const NO_SHARDS = "none";
const LANGUAGE_FRONTENDS = "language-frontends";
const NATIVE_FRONTENDS = "native-frontends";
const RELEASE_VALIDATION_GUARD = "crates/no-mistakes/tests/release_validate_cache.rs";

const nativeFixtureRoots = [
  "test-cases/codebase-analysis/swift-test-plan/fixture/",
  "test-cases/codebase-analysis/dotnet-test-plan/fixture/",
];

function isNativeFixture(path) {
  return nativeFixtureRoots.some((root) => path.startsWith(root));
}

function affectsEveryBenchmark(path) {
  return (
    path === "Cargo.toml" ||
    path === "Cargo.lock" ||
    path === ".github/workflows/ci.yml" ||
    path === ".github/scripts/benchmark-change-scope.cjs" ||
    path === ".github/scripts/benchmark-handoff.sh" ||
    path === "tests/js/benchmark-change-scope.test.js" ||
    path.startsWith("crates/") ||
    path.startsWith("fixtures/performance/") ||
    path.startsWith("fixtures/tsconfig/workspace-resolution/") ||
    path.startsWith(".cargo/") ||
    path.startsWith("rust-toolchain")
  );
}

function isModifiedReleaseValidationGuard({ status, path }) {
  // The benchmark's visible-file inventory includes this path, so additions,
  // deletions, type changes, and symlinks must still run the benchmarks.
  return (
    status === "M" &&
    path === RELEASE_VALIDATION_GUARD &&
    lstatSync(join(__dirname, "..", "..", path), { throwIfNoEntry: false })?.isFile()
  );
}

function benchmarkChangeScope(changes, forceAll = false) {
  if (forceAll) {
    return { changed: true, scope: ALL_SHARDS };
  }

  const selected = new Set();
  for (const change of changes) {
    const { path } = change;
    if (isModifiedReleaseValidationGuard(change)) {
      continue;
    }
    if (isNativeFixture(path)) {
      selected.add(NATIVE_FRONTENDS);
    } else if (path.startsWith("fixtures/lang-frontends/")) {
      selected.add(LANGUAGE_FRONTENDS);
      selected.add("frontend-cache-lifetimes");
    } else if (affectsEveryBenchmark(path)) {
      return { changed: true, scope: ALL_SHARDS };
    }
  }

  return {
    changed: selected.size > 0,
    scope: selected.size > 0 ? [...selected].sort().join(",") : NO_SHARDS,
  };
}

function readChangedFiles() {
  const fields = readFileSync(0).toString("utf8").split("\0");
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
  const forceAll = process.argv.includes("--all");
  const result = benchmarkChangeScope(forceAll ? [] : readChangedFiles(), forceAll);
  process.stdout.write(`changed=${result.changed}\nscope=${result.scope}\n`);
}

module.exports = { benchmarkChangeScope };
