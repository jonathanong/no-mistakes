const assert = require("node:assert/strict");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");
const { runProfile } = require("./native-compile-profile.cjs");

const saved = path.resolve(__dirname, "../../test-cases/native-compile-profile");
const manifest = "crates/no-mistakes/Cargo.toml";
const env = {
  RUNNER_OS: "macOS",
  NO_MISTAKES_BUILD_NAPI: "1",
  CARGO_INCREMENTAL: "0",
  RUSTFLAGS: "fixture",
};

function fixture(kind = "root") {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "native-compile-profile-"));
  fs.cpSync(path.join(saved, kind), root, { recursive: true });
  return { root, cleanup: () => fs.rmSync(root, { recursive: true, force: true }) };
}

function runner(
  root,
  {
    runtimeStatus = 0,
    archiveStatus = 0,
    throwRuntime = false,
    omitTarget = false,
    runtimeEvidence,
    archiveEvidence,
  } = {},
) {
  const calls = [];
  const original = fs.readFileSync(path.join(root, manifest));
  const run = (executable, args, options) => {
    assert.equal(executable, process.execPath);
    assert.equal(options.cwd, root);
    assert.equal(options.env.NO_MISTAKES_BUILD_NAPI, "1");
    assert.equal(options.env.CARGO_INCREMENTAL, "0");
    assert.equal(options.env.RUSTFLAGS, "fixture");
    assert.equal(options.env.CARGO_TARGET_DIR, path.join(root, "target"));
    const script = path.basename(args[0]);
    calls.push({ script, args: args.slice(1), target: options.env.CARGO_TARGET_DIR });
    if (script === "build-native.cjs") {
      assert.equal(
        fs.readFileSync(path.join(root, manifest), "utf8"),
        original
          .toString("utf8")
          .replace('crate-type = ["rlib"]', 'crate-type = ["rlib", "cdylib"]'),
      );
      if (throwRuntime) throw new Error("fixture runtime spawn failed");
      const destination = path.join(root, "target/native-build-diagnostics");
      fs.cpSync(path.join(saved, "diagnostics/runtime"), destination, { recursive: true });
      if (runtimeEvidence)
        fs.copyFileSync(
          path.join(saved, `evidence/${runtimeEvidence}.json`),
          path.join(destination, "cargo-units.json"),
        );
      if (runtimeStatus)
        fs.copyFileSync(
          path.join(saved, "failure-result.json"),
          path.join(destination, "result.json"),
        );
      return { status: runtimeStatus };
    }
    assert.equal(script, "native-test-archive.cjs");
    assert.deepEqual(args.slice(1), ["build"]);
    assert.deepEqual(fs.readFileSync(path.join(root, manifest)), original);
    fs.cpSync(
      path.join(saved, "diagnostics/archive"),
      path.join(root, "target/native-test-build-diagnostics"),
      { recursive: true },
    );
    if (archiveEvidence)
      fs.copyFileSync(
        path.join(saved, `evidence/${archiveEvidence}.json`),
        path.join(root, "target/native-test-build-diagnostics/0/cargo-units.json"),
      );
    if (archiveStatus || omitTarget)
      fs.rmSync(path.join(root, "target/native-test-build-diagnostics/3"), { recursive: true });
    return { status: archiveStatus };
  };
  return { calls, original, run };
}

test("runtime precedes the unchanged test archive in one target with Cargo reuse evidence", () => {
  const input = fixture();
  const fake = runner(input.root);
  let clock = 0;
  try {
    const report = runProfile({
      ...input,
      platform: "darwin",
      env,
      run: fake.run,
      now: () => clock++ * 1000,
    });
    assert.equal(report.exitCode, 0);
    assert.equal(report.manifestRestored, true);
    assert.deepEqual(
      fake.calls.map((call) => call.script),
      ["build-native.cjs", "native-test-archive.cjs"],
    );
    assert.equal(new Set(fake.calls.map((call) => call.target)).size, 1);
    assert.deepEqual(fs.readFileSync(path.join(input.root, manifest)), fake.original);
    assert.deepEqual(
      report.phases.map((phase) => phase.elapsedSeconds),
      [1, 1],
    );
    assert.equal(report.testBuilds.length, 4);
    assert.deepEqual(report.testBuilds[0].runtimeComparison.rootCounts.after, {
      fresh: 0,
      compiled: 3,
    });
    assert.deepEqual(report.testBuilds[1].previousComparison.rootCounts.after, {
      fresh: 2,
      compiled: 1,
    });
    assert.deepEqual(report.testBuilds[0].runtimeComparison.rootLibraryCounts.after, {
      fresh: 0,
      compiled: 1,
    });
    assert.deepEqual(report.testBuilds[1].previousComparison.rootLibraryCounts.after, {
      fresh: 1,
      compiled: 0,
    });
    assert.equal(report.testBuilds[0].runtimeComparison.rootComparison.matching.length, 1);
    assert.equal(report.runtime.metadata.rustc.stdout, "rustc fixture");
    assert.match(report.evidence, /signatures alone cannot prove reuse/);
    const persisted = JSON.parse(
      fs.readFileSync(path.join(input.root, "target/native-compile-profile/report.json"), "utf8"),
    );
    assert.deepEqual(persisted, report);
  } finally {
    input.cleanup();
  }
});

for (const [options, message] of [
  [{ runtimeEvidence: "empty" }, /empty/],
  [{ runtimeEvidence: "missing-runtime-cli" }, /missing runtime CLI/],
  [{ runtimeEvidence: "missing-runtime-library" }, /missing the normal root library/],
  [{ archiveEvidence: "empty" }, /empty/],
  [{ archiveEvidence: "missing-selected-harness" }, /missing selected harness/],
  [{ archiveEvidence: "missing-archive-library" }, /missing the normal root library/],
]) {
  test(`successful phases require expected Cargo evidence: ${JSON.stringify(options)}`, () => {
    const input = fixture();
    const fake = runner(input.root, options);
    try {
      const report = runProfile({ ...input, platform: "darwin", env, run: fake.run });
      assert.equal(report.exitCode, 1);
      assert.match(report.error || report.diagnosticError, message);
      assert.equal(report.manifestRestored, true);
      assert.deepEqual(fs.readFileSync(path.join(input.root, manifest)), fake.original);
      if (options.runtimeEvidence) assert.equal(fake.calls.length, 1);
      else {
        assert.equal(report.testBuilds.length, 4);
        assert.equal(report.testBuilds[0].runtimeComparison, undefined);
      }
    } finally {
      input.cleanup();
    }
  });
}

test("an actual failed runtime keeps empty partial evidence and its original failure", () => {
  const input = fixture();
  const fake = runner(input.root, { runtimeStatus: 17, runtimeEvidence: "empty" });
  try {
    const report = runProfile({ ...input, platform: "darwin", env, run: fake.run });
    assert.equal(report.exitCode, 17);
    assert.deepEqual(report.runtime.units.units, []);
    assert.equal(report.manifestRestored, true);
    assert.equal(report.testBuilds.length, 0);
  } finally {
    input.cleanup();
  }
});

test("an actual failed archive keeps empty partial evidence without comparisons", () => {
  const input = fixture();
  const fake = runner(input.root, { archiveStatus: 17, archiveEvidence: "empty" });
  try {
    const report = runProfile({ ...input, platform: "darwin", env, run: fake.run });
    assert.equal(report.exitCode, 17);
    assert.deepEqual(report.testBuilds[0].units.units, []);
    assert.equal(report.testBuilds[0].runtimeComparison, undefined);
    assert.equal(report.manifestRestored, true);
    assert.equal(report.testBuilds.length, 3);
  } finally {
    input.cleanup();
  }
});

for (const options of [{ runtimeStatus: 17 }, { throwRuntime: true }]) {
  test(`runtime failure restores manifest and keeps a failing partial report: ${JSON.stringify(options)}`, () => {
    const input = fixture();
    const fake = runner(input.root, options);
    try {
      const report = runProfile({ ...input, platform: "darwin", env, run: fake.run });
      assert.equal(report.exitCode, options.runtimeStatus || 1);
      assert.equal(report.manifestRestored, true);
      assert.equal(fake.calls.length, 1);
      assert.deepEqual(fs.readFileSync(path.join(input.root, manifest)), fake.original);
      const persisted = JSON.parse(
        fs.readFileSync(path.join(input.root, "target/native-compile-profile/report.json"), "utf8"),
      );
      assert.equal(persisted.exitCode, report.exitCode);
      assert.equal(persisted.phases.length, 1);
    } finally {
      input.cleanup();
    }
  });
}

test("test archive failures retain completed targets and the phase failure", () => {
  const input = fixture();
  const fake = runner(input.root, { archiveStatus: 17 });
  try {
    const report = runProfile({ ...input, platform: "darwin", env, run: fake.run });
    assert.equal(report.exitCode, 17);
    assert.equal(report.phases[1].exitCode, 17);
    assert.equal(report.testBuilds.length, 3);
    assert.equal(report.manifestRestored, true);
  } finally {
    input.cleanup();
  }
});

test("a successful archive without every target report fails evidence collection", () => {
  const input = fixture();
  const fake = runner(input.root, { omitTarget: true });
  try {
    const report = runProfile({ ...input, platform: "darwin", env, run: fake.run });
    assert.equal(report.exitCode, 1);
    assert.match(report.diagnosticError, /without every target/);
  } finally {
    input.cleanup();
  }
});

test("profiling guards reject incorrect host, N-API, incremental mode and manifest", () => {
  assert.throws(() => runProfile({ platform: "linux", env }), /requires macOS/);
  assert.throws(
    () => runProfile({ platform: "darwin", env: { ...env, NO_MISTAKES_BUILD_NAPI: "0" } }),
    /NO_MISTAKES_BUILD_NAPI=1/,
  );
  assert.throws(
    () => runProfile({ platform: "darwin", env: { ...env, CARGO_INCREMENTAL: "1" } }),
    /CARGO_INCREMENTAL=0/,
  );
  const input = fixture("invalid");
  try {
    assert.throws(() => runProfile({ ...input, platform: "darwin", env }), /exactly one original/);
  } finally {
    input.cleanup();
  }
});

for (const kind of ["preexisting-debug", "preexisting-rustc"]) {
  for (const external of [false, true]) {
    test(`cold target preflight preserves ${kind} in a ${external ? "external" : "root-local"} target`, () => {
      const input = fixture();
      const target = external ? fixture(kind) : null;
      const directory = target?.root || path.join(input.root, "target");
      if (!external) fs.cpSync(path.join(saved, kind), directory, { recursive: true });
      const original = fs.readFileSync(path.join(input.root, manifest));
      const marker = path.join(
        directory,
        kind === "preexisting-debug" ? "debug/existing.json" : ".rustc_info.json",
      );
      const existing = fs.readFileSync(marker);
      let calls = 0;
      try {
        assert.throws(
          () =>
            runProfile({
              ...input,
              platform: "darwin",
              env: external ? { ...env, CARGO_TARGET_DIR: directory } : env,
              run: () => {
                calls++;
              },
            }),
          external ? /requires the workspace target directory/ : /requires a cold Cargo target/,
        );
        assert.equal(calls, 0);
        assert.deepEqual(fs.readFileSync(path.join(input.root, manifest)), original);
        assert.deepEqual(fs.readFileSync(marker), existing);
        assert.equal(fs.existsSync(path.join(input.root, "target/native-compile-profile")), false);
      } finally {
        input.cleanup();
        target?.cleanup();
      }
    });
  }
}

const layouts = require("../../test-cases/native-compile-profile/target-layouts.json");
for (const layout of layouts.supported) {
  test(`target archive preflight accepts ${layout}`, () => {
    const input = fixture();
    const fake = runner(input.root);
    try {
      const report = runProfile({
        ...input,
        platform: "darwin",
        env: {
          ...env,
          CARGO_TARGET_DIR: layout.replace("<root>", input.root),
        },
        run: fake.run,
      });
      assert.equal(report.exitCode, 0);
      assert.equal(fake.calls.length, 2);
      assert.deepEqual(fs.readFileSync(path.join(input.root, manifest)), fake.original);
    } finally {
      input.cleanup();
    }
  });
}

for (const layout of layouts.unsupported) {
  test(`target archive preflight rejects unused ${layout} before mutation or execution`, () => {
    const input = fixture();
    const external = fixture();
    const directory = path.resolve(input.root, layout.replace("<external>", external.root));
    const original = fs.readFileSync(path.join(input.root, manifest));
    let calls = 0;
    try {
      assert.equal(fs.existsSync(directory), false);
      assert.throws(
        () =>
          runProfile({
            ...input,
            platform: "darwin",
            env: {
              ...env,
              CARGO_TARGET_DIR: directory,
            },
            run: () => {
              calls++;
            },
          }),
        /requires the workspace target directory/,
      );
      assert.equal(calls, 0);
      assert.equal(fs.existsSync(directory), false);
      assert.equal(fs.existsSync(path.join(input.root, "target")), false);
      assert.deepEqual(fs.readFileSync(path.join(input.root, manifest)), original);
    } finally {
      input.cleanup();
      external.cleanup();
    }
  });
}
