const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { test } = require("node:test");
const { collectCargoUnits, compareCargoUnits } = require("./native-cargo-units.cjs");

const fixture = path.resolve(__dirname, "../../test-cases/native-cargo-units");
function collect(name, options = {}) {
  return collectCargoUnits(fs.readFileSync(path.join(fixture, `${name}.jsonl`), "utf8"), {
    root: "/workspace",
    context: { command: ["cargo", "build"], rustc: "fixture", environment: { RUSTFLAGS: "" } },
    ...options,
  });
}

test("Cargo reports retain package, target, feature, full profile and fresh evidence", () => {
  const report = collect("runtime");
  assert.deepEqual(report.counts, { fresh: 0, compiled: 2 });
  assert.equal(report.units.length, 2);
  const root = report.units.find((unit) => unit.target.name === "no_mistakes");
  assert.equal(root.package_id, "path+file:///workspace/crates/no-mistakes#no-mistakes@0.1.0");
  assert.deepEqual(root.target, {
    name: "no_mistakes",
    kind: ["cdylib", "rlib"],
    crate_types: ["cdylib", "rlib"],
  });
  assert.deepEqual(root.features, ["default", "mermaid"]);
  assert.deepEqual(root.profile, {
    debug_assertions: true,
    debuginfo: 0,
    opt_level: "0",
    overflow_checks: true,
    test: false,
  });
  assert.equal(root.fresh, false);
  assert.deepEqual(root.filenames, ["target/debug/deps/libno_mistakes.rlib"]);
  assert.deepEqual(report.context.environment, { RUSTFLAGS: "" });
});

test("matching roots do not conceal differing dependency features or prove reuse", () => {
  const comparison = compareCargoUnits(collect("runtime"), collect("archive"));
  assert.deepEqual(comparison.rootCounts, {
    before: { fresh: 0, compiled: 1 },
    after: { fresh: 0, compiled: 2 },
  });
  assert.equal(comparison.rootComparison.matching.length, 1);
  assert.deepEqual(comparison.rootLibraryCounts, {
    before: { fresh: 0, compiled: 1 },
    after: { fresh: 0, compiled: 1 },
  });
  assert.equal(comparison.rootComparison.onlyAfter.length, 1);
  assert.equal(comparison.matching.length, 1);
  assert.deepEqual(comparison.matching[0].after, { fresh: 0, compiled: 1 });
  assert.deepEqual(comparison.onlyBefore[0].signature.features, ["std"]);
  const dependency = comparison.onlyAfter.find((entry) => entry.signature.target.name === "bstr");
  assert.deepEqual(dependency.signature.features, ["std", "unicode"]);
  assert.match(comparison.evidence, /do not establish reuse/);
});

test("normal root library counts exclude the CLI, build script and test harness", () => {
  const report = collect("root-kinds");
  const comparison = compareCargoUnits(report, report);
  assert.deepEqual(comparison.rootCounts.before, { fresh: 0, compiled: 4 });
  assert.deepEqual(comparison.rootLibraryCounts.before, { fresh: 0, compiled: 1 });
});

test("Cargo fresh evidence is retained for the sequential prototype invocation", () => {
  const after = collect("prototype", {
    context: { command: ["cargo", "test", "--no-run"], environment: { RUSTFLAGS: "" } },
  });
  const comparison = compareCargoUnits(collect("runtime"), after);
  assert.deepEqual(after.counts, { fresh: 2, compiled: 0 });
  assert.deepEqual(comparison.rootCounts.after, { fresh: 1, compiled: 0 });
  assert.equal(comparison.matching.length, 2);
  assert.deepEqual(comparison.rootComparison.matching[0].after, { fresh: 1, compiled: 0 });
  assert.deepEqual(comparison.context.after.command, ["cargo", "test", "--no-run"]);
});

test("profile and crate-type changes differ despite identical filenames", () => {
  const comparison = compareCargoUnits(collect("runtime"), collect("changed-profile-kind"));
  assert.equal(comparison.matching.length, 0);
  assert.equal(comparison.onlyBefore.length, 2);
  assert.equal(comparison.onlyAfter.length, 2);
  assert.equal(comparison.rootComparison.matching.length, 0);
});

test("target filenames use logical paths on Windows and outside the workspace", () => {
  const windows = collect("windows", {
    root: "C:\\workspace",
    targetDirectory: "C:\\cache\\native-target",
  });
  const external = collect("external-target", { targetDirectory: "/cache/native-target" });
  assert.deepEqual(windows.units[0].filenames, ["target/debug/deps/libno_mistakes.rlib"]);
  assert.deepEqual(external.units[0].filenames, windows.units[0].filenames);
});

test("recognized malformed artifacts fail with line context", () => {
  assert.throws(() => collect("incomplete"), /Incomplete Cargo compiler-artifact at line 1/);
  assert.throws(() => collect("malformed"), /Malformed Cargo compiler-artifact JSON at line 1/);
});

test("a caller can select an exact root package identity", () => {
  const runtime = collect("runtime");
  const comparison = compareCargoUnits(runtime, runtime, {
    rootPackageId: runtime.units.find((unit) => unit.target.name === "bstr").package_id,
  });
  assert.equal(comparison.rootComparison.matching[0].signature.target.name, "bstr");
  assert.equal(comparison.rootCounts.before.compiled, 1);
});
