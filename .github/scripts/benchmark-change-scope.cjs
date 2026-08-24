const { readFileSync } = require("node:fs");

const ALL_SHARDS = "all";
const NO_SHARDS = "none";
const LANGUAGE_FRONTENDS = "language-frontends";
const NATIVE_FRONTENDS = "native-frontends";

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
    path === "tests/js/benchmark-change-scope.test.js" ||
    path.startsWith("crates/") ||
    path.startsWith("fixtures/performance/") ||
    path.startsWith("fixtures/tsconfig/workspace-resolution/") ||
    path.startsWith(".cargo/") ||
    path.startsWith("rust-toolchain")
  );
}

function benchmarkChangeScope(paths, forceAll = false) {
  if (forceAll) {
    return { changed: true, scope: ALL_SHARDS };
  }

  const selected = new Set();
  for (const path of paths) {
    if (isNativeFixture(path)) {
      selected.add(NATIVE_FRONTENDS);
    } else if (path.startsWith("fixtures/lang-frontends/")) {
      selected.add(LANGUAGE_FRONTENDS);
    } else if (affectsEveryBenchmark(path)) {
      return { changed: true, scope: ALL_SHARDS };
    }
  }

  return {
    changed: selected.size > 0,
    scope: selected.size > 0 ? [...selected].sort().join(",") : NO_SHARDS,
  };
}

function readChangedPaths() {
  return readFileSync(0).toString("utf8").split("\0").filter(Boolean);
}

if (require.main === module) {
  const forceAll = process.argv.includes("--all");
  const result = benchmarkChangeScope(forceAll ? [] : readChangedPaths(), forceAll);
  process.stdout.write(`changed=${result.changed}\nscope=${result.scope}\n`);
}

module.exports = { benchmarkChangeScope };
