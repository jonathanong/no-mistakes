const assert = require("node:assert/strict");
const { spawn, spawnSync } = require("node:child_process");
const fs = require("node:fs");
const os = require("node:os");
const path = require("node:path");
const { test } = require("node:test");

const root = path.resolve(__dirname, "../..");
const wrapper = path.join(__dirname, "build-native.cjs");
const saved = path.join(root, "test-cases/native-build-diagnostics/fixture");
const diagnostics = "target/native-build-diagnostics";

test("native workflow retains failure artifacts, literal deadlines and shared build outputs", () => {
  const workflow = fs.readFileSync(path.join(root, ".github/workflows/ci.yml"), "utf8");
  const native = workflow.split("\n  native-build:\n")[1].split(/\n  [a-z][a-z-]*:\n/)[0];
  const upload = native
    .split("- name: Upload native build diagnostics\n")[1]
    .split("\n      - name:")[0];
  assert.match(
    upload,
    /if: \$\{\{ \(always\(\)\) && steps.build-artifact-source.outputs.reuse != 'true' \}\}/,
  );
  assert.match(upload, /timeout-minutes: 2/);
  assert.match(upload, /uses: actions\/upload-artifact@[a-f0-9]{40} # v\d/);
  assert.match(upload, /target\/native-build-diagnostics/);
  assert.match(
    native,
    /NATIVE_RUST_CACHE_HIT: \$\{\{ steps.native-workspace-cache.outputs.cache-hit == 'true' && 'true' \|\| 'false' \}\}/,
  );
  assert.match(native, /node \.github\/scripts\/build-native.cjs/);
  assert.match(native, /crate-type = \["rlib", "cdylib"\]/);
  assert.match(native, /cp "\$\{\{ matrix.addon \}\}" target\/debug\/no-mistakes.node/);
  assert.match(workflow, /node --test[^\n]*\.github\/scripts\/build-native.test.cjs/);
});

function assertSignalTermination(completed, platform = process.platform) {
  if (platform === "darwin") {
    assert.equal(completed.exitCode, 137);
    assert.equal(completed.signal, null);
  } else {
    assert.equal(completed.exitCode, null);
    assert.equal(completed.signal, "SIGKILL");
  }
}

test("signal diagnostics distinguish raw termination from macOS normalization", () => {
  const report = (name) =>
    JSON.parse(fs.readFileSync(path.join(saved, "../reports", `${name}.json`), "utf8"));
  assertSignalTermination(report("raw-sigkill"), "linux");
  assert.throws(
    () => assertSignalTermination(report("normalized-sigkill"), "linux"),
    assert.AssertionError,
  );
  assert.throws(
    () => assertSignalTermination(report("missing-signal"), "linux"),
    assert.AssertionError,
  );
  assertSignalTermination(report("normalized-sigkill"), "darwin");
});

function fixture(scenario, logFailure, temporaryParent = os.tmpdir()) {
  const directory = fs.realpathSync(
    fs.mkdtempSync(path.join(temporaryParent, "native-build-test-")),
  );
  fs.cpSync(saved, directory, { recursive: true });
  if (scenario === "prolific-cancel")
    fs.cpSync(path.join(saved, "../backpressure"), path.join(directory, "backpressure"), {
      recursive: true,
    });
  fs.chmodSync(path.join(directory, "bin/cargo"), 0o755);
  if (logFailure === "open")
    fs.cpSync(path.join(saved, "../log-failures/open"), path.join(directory, diagnostics), {
      recursive: true,
    });
  const search =
    scenario === "missing"
      ? ["empty-bin", path.dirname(process.execPath)]
      : ["bin", process.env.PATH];
  return {
    directory,
    options: {
      cwd: directory,
      encoding: "utf8",
      timeout: 20000,
      env: {
        ...process.env,
        PATH: search.map((entry) => path.resolve(directory, entry)).join(path.delimiter),
        NATIVE_BUILD_FIXTURE: scenario,
        ...(["midwrite", "stalled"].includes(logFailure)
          ? {
              NODE_OPTIONS: `--require=${path.join(saved, logFailure === "midwrite" ? "../log-failures/midwrite.cjs" : "../backpressure/stalled-log.cjs")}`,
            }
          : {}),
        NATIVE_RUST_CACHE_HIT: "true",
        PRIVATE_FIXTURE_SECRET: "private-fixture-token",
        CARGO_TARGET_DIR: path.join(directory, "cargo-target"),
      },
    },
    read(name) {
      return fs.readFileSync(path.join(directory, diagnostics, name), "utf8");
    },
    cleanup() {
      fs.rmSync(directory, { recursive: true, force: true });
    },
  };
}

test(
  "native wrapper normalizes units when its temporary parent has a symlink alias",
  { skip: process.platform === "win32" },
  () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-build-path-alias-"));
    fs.cpSync(path.join(saved, "../path-alias"), directory, {
      recursive: true,
      verbatimSymlinks: true,
    });
    const input = fixture("success", undefined, path.join(directory, "alias"));
    input.options.env.RUNNER_OS = "macOS";
    try {
      const result = spawnSync(process.execPath, [wrapper], input.options);
      assert.equal(result.status, 0, result.stderr);
      const units = JSON.parse(input.read("cargo-units.json"));
      assert.deepEqual(units.counts, { fresh: 0, compiled: 1 });
      assert.deepEqual(units.units[0].filenames, [
        "target/debug/libno_mistakes.rlib",
        "target/debug/libno_mistakes.so",
      ]);
      assert.equal(input.directory, fs.realpathSync(input.directory));
    } finally {
      input.cleanup();
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);

for (const scenario of ["success", "interleaved"])
  test(
    `native runtime wrapper retains macOS profiles and complete Cargo units with ${scenario} output`,
    { skip: process.platform === "win32" },
    () => {
      const input = fixture(scenario);
      input.options.env.RUNNER_OS = "macOS";
      try {
        const result = spawnSync(process.execPath, [wrapper], input.options);
        assert.equal(result.status, 0, result.stderr);
        assert.deepEqual(JSON.parse(input.read("metadata.json")).command, [
          "cargo",
          "build",
          ...require("../../test-cases/native-build-diagnostics/reports/macos-profile.json"),
          "--locked",
          "-p",
          "no-mistakes",
          "--timings",
          "--verbose",
          "--message-format=json-render-diagnostics",
        ]);
        const units = JSON.parse(input.read("cargo-units.json"));
        assert.deepEqual(units.counts, { fresh: 0, compiled: 1 });
        assert.deepEqual(units.units[0].target.crate_types, ["cdylib", "rlib"]);
        assert.deepEqual(units.units[0].filenames, [
          "target/debug/libno_mistakes.rlib",
          "target/debug/libno_mistakes.so",
        ]);
        assert.deepEqual(units.context.command, JSON.parse(input.read("metadata.json")).command);
        if (scenario === "interleaved")
          assert.match(input.read("cargo.log"), /fixture interleaved compiler stderr/);
      } finally {
        input.cleanup();
      }
    },
  );

for (const [scenario, status] of [
  ["success", 0],
  ["failure", 17],
  ["missing", 1],
  ["signal", 137],
  ["exit-137", 137],
]) {
  test(
    `native build retains diagnostics and preserves ${scenario} status`,
    { skip: process.platform === "win32" },
    () => {
      const input = fixture(scenario);
      try {
        const result = spawnSync(process.execPath, [wrapper], input.options);
        if (scenario === "missing") assert.notEqual(result.status, 0, result.stderr);
        else assert.equal(result.status, status, result.stderr);
        const metadata = input.read("metadata.json");
        assert.ok(!metadata.includes("private-fixture-token"));
        assert.equal(JSON.parse(metadata).environment.NATIVE_RUST_CACHE_HIT, "true");
        const completed = JSON.parse(input.read("result.json"));
        assert.ok(completed.elapsedSeconds >= 0);
        assert.ok(input.read("processes.jsonl").trim().split("\n").length >= 2);
        if (scenario === "missing") {
          if (process.platform === "darwin") assert.match(result.stderr, /No such file|not found/i);
          else assert.match(input.read("spawn-error.txt"), /ENOENT/);
          for (const line of input.read("processes.jsonl").trim().split("\n")) {
            const record = JSON.parse(line);
            assert.equal(record.status, null);
            assert.equal(typeof record.error, "string");
            assert.match(record.error, /ENOENT/);
          }
        } else {
          if (scenario === "signal") assertSignalTermination(completed);
          else assert.equal(completed.exitCode, status);
          if (scenario === "exit-137") assert.equal(completed.signal, null);
          assert.match(result.stdout, /fixture compiler stdout/);
          assert.match(result.stderr, /fixture compiler stderr/);
          assert.match(input.read("cargo.log"), /fixture compiler stderr/);
          assert.equal(
            input.read("cargo-timings/cargo-timing.html"),
            fs.readFileSync(path.join(saved, "timing.html"), "utf8"),
          );
          if (scenario === "success") {
            for (const line of input.read("processes.jsonl").trim().split("\n")) {
              const record = JSON.parse(line);
              assert.equal(record.status, 0);
              assert.equal(Object.hasOwn(record, "error"), false);
            }
          }
        }
      } finally {
        input.cleanup();
      }
    },
  );
}

for (const logFailure of [undefined, "open", "midwrite", "stalled"]) {
  test(
    `native build forwards cancellation with ${logFailure || "working"} log`,
    { skip: process.platform === "win32" },
    async () => {
      const input = fixture(logFailure === "stalled" ? "prolific-cancel" : "cancel", logFailure);
      try {
        const child = spawn(process.execPath, [wrapper], {
          ...input.options,
          killSignal: "SIGKILL",
        });
        let cancelled = false;
        const cancel = () => {
          if (cancelled) return;
          cancelled = true;
          child.kill("SIGTERM");
        };
        let stdout = "";
        child.stdout.on("data", (chunk) => {
          stdout += chunk;
          if (stdout.includes("fixture-ready")) cancel();
        });
        let stderr = "";
        child.stderr.on("data", (chunk) => {
          stderr += chunk;
          if (logFailure === "stalled" && stderr.includes("fixture-log-stalled")) cancel();
        });
        const status = await new Promise((resolve, reject) => {
          child.on("error", reject);
          child.on("close", resolve);
        });
        assert.equal(status, 143, stderr);
        assert.equal(JSON.parse(input.read("cancellation.json")).signal, "SIGTERM");
        assert.equal(JSON.parse(input.read("result.json")).cancelledSignal, "SIGTERM");
        if (logFailure && logFailure !== "stalled")
          assert.match(input.read("log-error.txt"), logFailure === "open" ? /EISDIR/ : /ENOSPC/);
      } finally {
        input.cleanup();
      }
    },
  );
}

test("sample writer retains a timed-out process probe error", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-probe-sample-"));
  try {
    const result = spawnSync(
      process.execPath,
      [
        path.join(root, "test-cases/native-build-diagnostics/sample-probe-failure.cjs"),
        "timeout",
      ],
      { cwd: directory, encoding: "utf8", timeout: 20000 },
    );
    assert.equal(result.status, 0, result.stderr);
    const record = JSON.parse(
      fs.readFileSync(
        path.join(directory, "target/native-build-diagnostics/processes.jsonl"),
        "utf8",
      ),
    );
    assert.equal(record.status, null);
    assert.equal(typeof record.error, "string");
    assert.match(record.error, /ETIMEDOUT/);
    assert.equal(Object.hasOwn(record, "stderr"), true);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

require("./native-build-output.test.cjs");
require("./native-build-output-edge.test.cjs");
require("./native-build-probes.test.cjs");

for (const logFailure of ["open", "midwrite"]) {
  for (const [scenario, status] of [
    ["success", 0],
    ["failure", 17],
    ["signal", 137],
    ["exit-137", 137],
  ]) {
    test(
      `native build preserves ${scenario} status after ${logFailure} log failure`,
      { skip: process.platform === "win32" },
      () => {
        const input = fixture(scenario, logFailure);
        try {
          const result = spawnSync(process.execPath, [wrapper], input.options);
          assert.equal(result.status, status, result.stderr);
          const completed = JSON.parse(input.read("result.json"));
          if (scenario === "signal") assertSignalTermination(completed);
          else assert.equal(completed.exitCode, status);
          if (scenario === "exit-137") assert.equal(completed.signal, null);
          assert.match(input.read("log-error.txt"), logFailure === "open" ? /EISDIR/ : /ENOSPC/);
          assert.match(result.stdout, /fixture compiler stdout/);
          assert.match(result.stderr, /fixture compiler stderr/);
          if (logFailure === "midwrite") assert.ok(input.read("cargo.log").length > 0);
        } finally {
          input.cleanup();
        }
      },
    );
  }
}

require("./native-rust-tests.test.cjs");
require("./native-test-artifacts.test.cjs");
require("./native-api-shards.test.cjs");
require("./native-rust-partition.test.cjs");
require("./build-artifact-source.test.cjs");

require("./build-artifact-workflow.test.cjs");
