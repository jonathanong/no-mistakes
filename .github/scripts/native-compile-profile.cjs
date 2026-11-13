const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const { compareCargoUnits } = require("./native-cargo-units.cjs");
const { targets } = require("./native-rust-tests.cjs");

function runProfile({
  root = process.cwd(),
  platform = process.platform,
  env = process.env,
  run = spawnSync,
  now = Date.now,
} = {}) {
  if (platform !== "darwin" || env.RUNNER_OS !== "macOS")
    throw new Error("native compile profiling requires macOS");
  if (env.NO_MISTAKES_BUILD_NAPI !== "1" || env.CARGO_INCREMENTAL !== "0")
    throw new Error(
      "native compile profiling requires NO_MISTAKES_BUILD_NAPI=1 and CARGO_INCREMENTAL=0",
    );
  const manifest = path.join(root, "crates/no-mistakes/Cargo.toml");
  const original = fs.readFileSync(manifest);
  const before = 'crate-type = ["rlib"]';
  const source = original.toString("utf8");
  if (source.split(before).length !== 2)
    throw new Error('native compile profiling requires exactly one original crate-type = ["rlib"]');
  const environment = {
    ...env,
    CARGO_TARGET_DIR: path.resolve(root, env.CARGO_TARGET_DIR || "target"),
  };
  for (const marker of ["debug", ".rustc_info.json"]) {
    const existing = path.join(environment.CARGO_TARGET_DIR, marker);
    if (fs.existsSync(existing))
      throw new Error(
        `native compile profiling requires a cold Cargo target; ${existing} already exists. Use a new CARGO_TARGET_DIR.`,
      );
  }
  const output = path.join(root, "target/native-compile-profile");
  fs.mkdirSync(output, { recursive: true });
  for (const directory of ["native-build-diagnostics", "native-test-build-diagnostics"])
    fs.rmSync(path.join(root, "target", directory), { recursive: true, force: true });
  const report = {
    schemaVersion: 1,
    order: "runtime-then-original-test-archive",
    targetDirectory: environment.CARGO_TARGET_DIR,
    evidence:
      "Only Cargo fresh=true records reuse in this sequential build. Reported signatures alone cannot prove reuse or fingerprint compatibility.",
    phases: [],
    manifestRestored: false,
    runtime: null,
    testBuilds: [],
    exitCode: 1,
  };
  const started = now();
  function save() {
    report.elapsedSeconds = (now() - started) / 1000;
    fs.writeFileSync(path.join(output, "report.json"), `${JSON.stringify(report, null, 2)}\n`);
  }
  function phase(name, script, args = []) {
    const began = now();
    let result;
    try {
      result = run(process.execPath, [path.join(__dirname, script), ...args], {
        cwd: root,
        env: environment,
        stdio: "inherit",
      });
    } catch (error) {
      result = { status: null, error };
    }
    const entry = {
      name,
      elapsedSeconds: (now() - began) / 1000,
      exitCode: result.status ?? null,
      signal: result.signal ?? null,
      error: result.error?.message ?? null,
    };
    report.phases.push(entry);
    save();
    return result.error || result.signal ? 1 : (result.status ?? 1);
  }
  function readReport(directory) {
    const read = (name) => JSON.parse(fs.readFileSync(path.join(root, directory, name), "utf8"));
    return {
      units: read("cargo-units.json"),
      metadata: read("metadata.json"),
      result: read("result.json"),
    };
  }
  try {
    let runtimeStatus;
    try {
      fs.writeFileSync(manifest, source.replace(before, 'crate-type = ["rlib", "cdylib"]'));
      runtimeStatus = phase("runtime", "build-native.cjs");
    } finally {
      fs.writeFileSync(manifest, original);
      report.manifestRestored = fs.readFileSync(manifest).equals(original);
    }
    if (!report.manifestRestored)
      throw new Error("original manifest was not restored before test archive");
    report.exitCode = runtimeStatus;
    try {
      report.runtime = readReport("target/native-build-diagnostics");
    } catch (error) {
      if (!runtimeStatus) throw error;
      report.diagnosticError = error.message;
    }
    if (runtimeStatus) return report;
    report.exitCode = phase("test-archive", "native-test-archive.cjs", ["build"]);
    let previous = report.runtime.units;
    for (const [index, target] of targets(environment.RUNNER_OS).entries()) {
      const directory = `target/native-test-build-diagnostics/${index}`;
      if (!fs.existsSync(path.join(root, directory))) continue;
      try {
        const build = readReport(directory);
        report.testBuilds.push({
          index,
          selector: target.selector,
          ...build,
          runtimeComparison: compareCargoUnits(report.runtime.units, build.units),
          previousComparison: compareCargoUnits(previous, build.units),
        });
        previous = build.units;
      } catch (error) {
        report.diagnosticError = error.message;
        if (!report.exitCode) report.exitCode = 1;
      }
    }
    if (!report.exitCode && report.testBuilds.length !== targets(environment.RUNNER_OS).length) {
      report.exitCode = 1;
      report.diagnosticError = "test archive completed without every target's Cargo unit report";
    }
  } catch (error) {
    if (!report.exitCode) report.exitCode = 1;
    report.error = error.message;
  } finally {
    save();
  }
  return report;
}

if (require.main === module) {
  try {
    process.exitCode = runProfile().exitCode;
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}

module.exports = { runProfile };
