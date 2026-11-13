const assert = require("node:assert/strict");
const { spawn, spawnSync } = require("node:child_process");
const {
  existsSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
} = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { test } = require("node:test");

const root = join(__dirname, "../..");
const workflow = readFileSync(join(root, ".github/workflows/ci.yml"), "utf8");
const step = workflow.split("- name: Run Rust tests for coverage")[1].split("- name:")[0];
const pattern = step.match(/LLVM_PROFILE_FILE_NAME: (\S+)/)?.[1];
test("coverage subprocesses share a bounded binary-signature pool", () => {
  assert.match(pattern, /^no-mistakes-%[1-9]m\.profraw$/);
  const job = (
    workflow.split("\n  rust-coverage-build:")[1] || workflow.split("\n  tests:")[1]
  ).split(/^  [\w-]+:/m)[0];
  assert.match(job, /components: llvm-tools-preview/);
  assert.match(job, /run: node --test [^\n]*\.github\/scripts\/rust-profile-pooling\.test\.cjs/);
});

const rustc = (...args) => spawnSync("rustc", args, { encoding: "utf8" });
const sysroot = rustc("--print", "sysroot").stdout?.trim();
const host = rustc("-vV").stdout?.match(/^host: (.+)$/m)?.[1];
const suffix = process.platform === "win32" ? ".exe" : "";
const tools = sysroot && host ? join(sysroot, "lib/rustlib", host, "bin") : "";
const profdata = join(tools, `llvm-profdata${suffix}`);
const cov = join(tools, `llvm-cov${suffix}`);
const run = (command, args) => {
  const result = spawnSync(command, args, { encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
};

test(
  "pooled concurrent processes retain every branch and execution count",
  {
    timeout: 60_000,
  },
  async () => {
    assert.ok(existsSync(profdata) && existsSync(cov), "requires rustup llvm-tools-preview");
    const temp = mkdtempSync(join(tmpdir(), "nm-profile-pooling-"));
    try {
      const binary = join(temp, `probe${suffix}`);
      run("rustc", [
        "-C",
        "instrument-coverage",
        join(root, "test-cases/rust-profile-preservation/pooling.rs"),
        "-o",
        binary,
      ]);
      const reports = [];
      for (const name of ["probe-%p-%9m.profraw", pattern]) {
        const directory = join(temp, reports.length.toString());
        mkdirSync(directory);
        for (let batch = 0; batch < 10; batch++) {
          await Promise.all(
            Array.from(
              { length: 10 },
              (_, index) =>
                new Promise((resolve, reject) => {
                  const child = spawn(binary, [index % 2 ? "left" : "right"], {
                    env: { ...process.env, LLVM_PROFILE_FILE: join(directory, name) },
                    stdio: "ignore",
                  });
                  child.on("error", reject);
                  child.on("exit", (code) =>
                    code === 0 ? resolve() : reject(new Error(`probe exit ${code}`)),
                  );
                }),
            ),
          );
        }
        const raw = readdirSync(directory).filter((file) => file.endsWith(".profraw"));
        if (reports.length === 0) assert.equal(raw.length, 100);
        else assert.ok(raw.length > 0 && raw.length <= Number(pattern.match(/%(\d)m/)[1]));
        const merged = join(directory, "merged.profdata");
        run(profdata, [
          "merge",
          "-sparse",
          ...raw.map((file) => join(directory, file)),
          "-o",
          merged,
        ]);
        reports.push(JSON.parse(run(cov, ["export", binary, "-instr-profile", merged])).data);
      }
      assert.deepEqual(reports[1], reports[0]);
      assert.equal(reports[1][0].functions[0].count, 100);
    } finally {
      rmSync(temp, { recursive: true, force: true });
    }
  },
);
