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
  const native = workflow.split("\n  native-tests:\n")[1].split(/\n  [a-z][a-z-]*:\n/)[0];
  const upload = native
    .split("- name: Upload native build diagnostics\n")[1]
    .split("\n      - name:")[0];
  assert.match(upload, /if: \$\{\{ always\(\) \}\}/);
  assert.match(upload, /timeout-minutes: 2/);
  assert.match(upload, /uses: actions\/upload-artifact@[a-f0-9]{40} # v\d/);
  assert.match(upload, /path: target\/native-build-diagnostics/);
  assert.match(
    native,
    /NATIVE_RUST_CACHE_HIT: \$\{\{ steps.native-rust-cache.outputs.cache-hit \}\}/,
  );
  assert.match(native, /NO_MISTAKES_BUILD_NAPI=1 node \.github\/scripts\/build-native.cjs/);
  assert.match(native, /crate-type = \["rlib", "cdylib"\]/);
  assert.match(native, /cp "\$\{\{ matrix.addon \}\}" target\/debug\/no-mistakes.node/);
  assert.match(workflow, /node --test[^\n]*\.github\/scripts\/build-native.test.cjs/);
});

function fixture(scenario) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "native-build-test-"));
  fs.cpSync(saved, directory, { recursive: true });
  fs.chmodSync(path.join(directory, "bin/cargo"), 0o755);
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

for (const [scenario, status] of [
  ["success", 0],
  ["failure", 17],
  ["missing", 1],
  ["signal", 137],
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
        } else {
          if (scenario === "signal")
            assert.ok(completed.signal === "SIGKILL" || completed.exitCode === 137);
          else assert.equal(completed.exitCode, status);
          assert.match(result.stdout, /fixture compiler stdout/);
          assert.match(result.stderr, /fixture compiler stderr/);
          assert.match(input.read("cargo.log"), /fixture compiler stderr/);
          assert.equal(
            input.read("cargo-timings/cargo-timing.html"),
            fs.readFileSync(path.join(saved, "timing.html"), "utf8"),
          );
        }
      } finally {
        input.cleanup();
      }
    },
  );
}

test(
  "native build forwards cancellation and retains a failing terminal record",
  { skip: process.platform === "win32" },
  async () => {
    const input = fixture("cancel");
    try {
      const child = spawn(process.execPath, [wrapper], input.options);
      let stdout = "";
      child.stdout.on("data", (chunk) => {
        stdout += chunk;
        if (stdout.includes("fixture-ready")) child.kill("SIGTERM");
      });
      let stderr = "";
      child.stderr.on("data", (chunk) => {
        stderr += chunk;
      });
      const status = await new Promise((resolve, reject) => {
        child.on("error", reject);
        child.on("close", resolve);
      });
      assert.equal(status, 143, stderr);
      assert.equal(JSON.parse(input.read("cancellation.json")).signal, "SIGTERM");
      assert.equal(JSON.parse(input.read("result.json")).cancelledSignal, "SIGTERM");
    } finally {
      input.cleanup();
    }
  },
);
