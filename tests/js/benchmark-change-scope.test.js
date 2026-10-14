const assert = require("node:assert/strict");
const { spawnSync } = require("node:child_process");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");

const selector = join(__dirname, "..", "..", ".github", "scripts", "benchmark-change-scope.cjs");
const { benchmarkChangeScope } = require(selector);
const modified = (paths) => paths.map((path) => ({ status: "M", path }));

test("routes native fixture-only changes to native frontends", () => {
  assert.deepEqual(
    benchmarkChangeScope(
      modified([
        "test-cases/codebase-analysis/swift-test-plan/fixture/swift-clients/core/Package.swift",
        "test-cases/codebase-analysis/dotnet-test-plan/fixture/dotnet-clients/App.sln",
        "docs/architecture.md",
      ]),
    ),
    { changed: true, scope: "native-frontends" },
  );
});

test("routes general language fixtures independently from native fixtures", () => {
  assert.deepEqual(
    benchmarkChangeScope(modified(["fixtures/lang-frontends/go-http/handlers.go"])),
    {
      changed: true,
      scope: "language-frontends",
    },
  );
  assert.deepEqual(
    benchmarkChangeScope(
      modified([
        "fixtures/lang-frontends/go-http/handlers.go",
        "test-cases/codebase-analysis/swift-test-plan/fixture/swift-clients/core/Package.swift",
      ]),
    ),
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
    ".github/scripts/benchmark-change-scope.cjs",
    ".github/scripts/benchmark-handoff.sh",
    "tests/js/benchmark-change-scope.test.js",
  ]) {
    assert.deepEqual(benchmarkChangeScope(modified([path])), { changed: true, scope: "all" });
  }
});

test("ignores unrelated changes and supports an explicit full fallback", () => {
  assert.deepEqual(benchmarkChangeScope(modified(["docs/architecture.md"])), {
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
  assert.match(workflow, /git diff --no-renames --name-status -z/);
});

test("only a modified regular release validation guard is unrelated to benchmarks", () => {
  const guard = "crates/no-mistakes/tests/release_validate_cache.rs";
  assert.deepEqual(benchmarkChangeScope(modified([guard])), { changed: false, scope: "none" });
  for (const status of ["A", "D", "T"]) {
    assert.deepEqual(benchmarkChangeScope([{ status, path: guard }]), {
      changed: true,
      scope: "all",
    });
  }
  assert.deepEqual(benchmarkChangeScope([{ path: guard }]), { changed: true, scope: "all" });
  assert.deepEqual(benchmarkChangeScope(modified(["crates/no-mistakes/tests/other.rs"])), {
    changed: true,
    scope: "all",
  });
  assert.deepEqual(benchmarkChangeScope(modified([guard, "crates/no-mistakes/src/lib.rs"])), {
    changed: true,
    scope: "all",
  });
  assert.deepEqual(
    benchmarkChangeScope(modified([guard, "fixtures/lang-frontends/go-http/handlers.go"])),
    { changed: true, scope: "language-frontends" },
  );
  assert.deepEqual(
    benchmarkChangeScope([
      { status: "D", path: guard },
      { status: "A", path: "crates/no-mistakes/tests/renamed.rs" },
    ]),
    { changed: true, scope: "all" },
  );
});

test("CLI reads NUL-delimited statuses and fails closed on malformed input", () => {
  const guard = "crates/no-mistakes/tests/release_validate_cache.rs";
  const run = (input) => spawnSync(process.execPath, [selector], { input, encoding: "utf8" });
  const modifiedGuard = run(`M\0${guard}\0`);
  assert.equal(modifiedGuard.status, 0);
  assert.equal(modifiedGuard.stdout, "changed=false\nscope=none\n");
  const deletedGuard = run(`D\0${guard}\0`);
  assert.equal(deletedGuard.status, 0);
  assert.equal(deletedGuard.stdout, "changed=true\nscope=all\n");
  for (const malformed of [`M\0${guard}`, `Z\0${guard}\0`, `M\0${guard}\0D\0`]) {
    assert.notEqual(run(malformed).status, 0);
  }
});
