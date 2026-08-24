const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const { benchmarkChangeScope } = require(
  join(__dirname, "..", "..", ".github", "scripts", "benchmark-change-scope.cjs"),
);

test("routes native fixture-only changes to native frontends", () => {
  assert.deepEqual(
    benchmarkChangeScope([
      "test-cases/codebase-analysis/swift-test-plan/fixture/swift-clients/core/Package.swift",
      "test-cases/codebase-analysis/dotnet-test-plan/fixture/dotnet-clients/App.sln",
      "docs/architecture.md",
    ]),
    { changed: true, scope: "native-frontends" },
  );
});

test("routes general language fixtures independently from native fixtures", () => {
  assert.deepEqual(benchmarkChangeScope(["fixtures/lang-frontends/go-http/handlers.go"]), {
    changed: true,
    scope: "language-frontends",
  });
  assert.deepEqual(
    benchmarkChangeScope([
      "fixtures/lang-frontends/go-http/handlers.go",
      "test-cases/codebase-analysis/swift-test-plan/fixture/swift-clients/core/Package.swift",
    ]),
    { changed: true, scope: "language-frontends,native-frontends" },
  );
});

test("conservatively runs every shard for shared benchmark inputs", () => {
  for (const path of [
    "Cargo.lock",
    "crates/no-mistakes/src/lib.rs",
    "fixtures/performance/graph-gates/package.json",
    "fixtures/tsconfig/workspace-resolution/tsconfig.json",
    ".github/workflows/ci.yml",
  ]) {
    assert.deepEqual(benchmarkChangeScope([path]), { changed: true, scope: "all" });
  }
});

test("ignores unrelated changes and supports an explicit full fallback", () => {
  assert.deepEqual(benchmarkChangeScope(["docs/architecture.md"]), {
    changed: false,
    scope: "none",
  });
  assert.deepEqual(benchmarkChangeScope([], true), { changed: true, scope: "all" });
});

test("change discovery preserves both sides of fixture renames", () => {
  const workflow = readFileSync(
    join(__dirname, "..", "..", ".github", "workflows", "ci.yml"),
    "utf8",
  );
  assert.match(workflow, /git diff --no-renames --name-only -z/);
});
