const assert = require("node:assert/strict");
const { mkdtemp, mkdir, readFile, rm, writeFile } = require("node:fs/promises");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const test = globalThis.test || require("node:test").test;

const {
  boundedDiagnostic,
  main,
  parsePlanningImpactArgs,
  runPlanningImpact,
} = require("../planning-impact-cli");
const {
  firstSubcommand,
  launchNative,
  main: launcherMain,
  planningImpactArgs,
} = require("../bin/no-mistakes.js");

test("parses the planning-impact artifact command and its invocation controls", () => {
  assert.deepEqual(
    parsePlanningImpactArgs(
      [
        "--changed-files",
        "changed.txt",
        "--output-dir=output",
        "--broad",
        "--timeout",
        "0",
        "--lock-timeout",
        "12",
        "--fail-on-lock",
        "-j",
        "2",
        "--profile",
        "ci",
      ],
      "/repo",
    ),
    {
      root: "/repo",
      changedFilesManifest: "changed.txt",
      outputDirectory: "output",
      broad: true,
      timeout: 0,
      lockTimeout: 12,
      failOnLock: true,
      jobs: 2,
      profile: "ci",
    },
  );
});

test("reports help and rejects missing or unknown planning-impact arguments", () => {
  assert.equal(parsePlanningImpactArgs(["--help"], "/repo"), null);
  assert.throws(
    () => parsePlanningImpactArgs(["--changed-files", "changed.txt"], "/repo"),
    /usage: no-mistakes planning-impact/,
  );
  assert.throws(
    () => parsePlanningImpactArgs(["--changed-files", "x", "--output-dir", "y", "--wat"], "/repo"),
    /Unknown option/,
  );
  assert.throws(
    () =>
      parsePlanningImpactArgs(["--changed-files", "x", "--output-dir", "y", "--jobs=-1"], "/repo"),
    /jobs must be a non-negative integer/,
  );
  assert.throws(
    () =>
      parsePlanningImpactArgs(
        ["--changed-files", "x", "--output-dir", "y", "--timeout=9007199254740992"],
        "/repo",
      ),
    /timeout must be a safe integer/,
  );
  assert.throws(
    () =>
      parsePlanningImpactArgs(
        ["--changed-files", "x", "--output-dir", "y", "--profile", "dev"],
        "/repo",
      ),
    /profile must be "ci"/,
  );
  assert.throws(
    () =>
      parsePlanningImpactArgs(["--changed-files", "x", "--output-dir", "y", "unexpected"], "/repo"),
    /Unexpected argument/,
  );
});

test("does not call the writer for help and renders no success output", async () => {
  let calls = 0;
  assert.equal(
    await runPlanningImpact(["--help"], "/repo", async () => {
      calls += 1;
    }),
    null,
  );
  assert.equal(calls, 0);
  let stdout = "";
  await main(
    ["--changed-files", "changed.txt", "--output-dir", "output"],
    "/repo",
    { stdout: { write: (value) => (stdout += value) } },
    async () => {},
  );
  assert.equal(stdout, "");
  assert.equal(boundedDiagnostic("failure"), "failure\n");
});

test("loads the public writer lazily when main receives no writer", async () => {
  const indexPath = require.resolve("../index");
  const previous = require.cache[indexPath];
  const calls = [];
  require.cache[indexPath] = {
    exports: { writePlanningImpactArtifacts: async (options) => calls.push(options) },
  };
  try {
    await main(["--changed-files", "changed.txt", "--output-dir", "output"], "/repo", {
      stdout: { write() {} },
    });
    assert.deepEqual(calls, [
      {
        root: "/repo",
        changedFilesManifest: "changed.txt",
        outputDirectory: "output",
        broad: false,
      },
    ]);
  } finally {
    if (previous) require.cache[indexPath] = previous;
    else delete require.cache[indexPath];
  }
});

test("runs the public writer with cwd as root", async () => {
  const calls = [];
  await runPlanningImpact(
    ["--changed-files", "changed.txt", "--output-dir", "output", "--broad"],
    "/repo",
    async (options) => calls.push(options),
  );
  assert.deepEqual(calls, [
    {
      root: "/repo",
      changedFilesManifest: "changed.txt",
      outputDirectory: "output",
      broad: true,
    },
  ]);
});

test("prints help before loading the native addon", async () => {
  let stdout = "";
  await main(["--help"], "/repo", { stdout: { write: (value) => (stdout += value) } });
  assert.match(stdout, /usage: no-mistakes planning-impact/);
});

test("bounds CLI diagnostics without splitting UTF-8 characters", () => {
  const detail = "é".repeat(4_096);
  assert.ok(Buffer.byteLength(boundedDiagnostic(new Error(detail))) <= 4_096);
  assert.equal(boundedDiagnostic(new Error(detail)).includes("\ufffd"), false);
});

test("recognizes planning-impact only as the first actual subcommand", () => {
  assert.equal(firstSubcommand(["--timeout", "5", "planning-impact"]), "planning-impact");
  assert.equal(firstSubcommand(["--jobs=2", "planning-impact"]), "planning-impact");
  assert.equal(firstSubcommand(["--fail-on-lock", "planning-impact"]), "planning-impact");
  assert.equal(firstSubcommand(["--", "planning-impact"]), undefined);
  assert.equal(firstSubcommand([]), undefined);
  assert.equal(firstSubcommand(["--timings", "planning-impact"]), "--timings");
  assert.equal(firstSubcommand(["--wat", "planning-impact"]), "--wat");
  assert.equal(firstSubcommand(["dependencies", "planning-impact"]), "dependencies");
  assert.deepEqual(
    planningImpactArgs([
      "--timeout",
      "5",
      "planning-impact",
      "--changed-files",
      "x",
      "--output-dir",
      "y",
    ]),
    ["--timeout", "5", "--changed-files", "x", "--output-dir", "y"],
  );
});

test("forwards a successful planning command and native command unchanged", async () => {
  const planningCalls = [];
  await launcherMain(["--profile", "ci", "planning-impact", "--help"], {
    runPlanning: async (args) => planningCalls.push(args),
  });
  assert.deepEqual(planningCalls, [["--profile", "ci", "--help"]]);
  const nativeCalls = [];
  await launcherMain(["dependencies", "src/a.ts"], {
    launchNativeFn: (args) => nativeCalls.push(args),
  });
  await launcherMain(["--", "planning-impact"], {
    launchNativeFn: (args) => nativeCalls.push(args),
  });
  assert.deepEqual(nativeCalls, [
    ["dependencies", "src/a.ts"],
    ["--", "planning-impact"],
  ]);
});

test("runs the default planning command through the launcher", async () => {
  let stdout = "";
  await launcherMain(["planning-impact", "--help"], {
    io: { stdout: { write: (value) => (stdout += value) } },
  });
  assert.match(stdout, /usage: no-mistakes planning-impact/);
});

test("bounds a rejected planning command at the launcher boundary", async () => {
  let stderr = "";
  const io = { exitCode: undefined, stderr: { write: (value) => (stderr += value) } };
  await launcherMain(["planning-impact"], {
    io,
    runPlanning: async () => {
      throw new Error("é".repeat(4_096));
    },
  });
  assert.equal(io.exitCode, 1);
  assert.ok(Buffer.byteLength(stderr) <= 4_096);
  assert.equal(stderr.includes("\ufffd"), false);
});

test("delegates native arguments unchanged and propagates its exit code", async () => {
  const calls = [];
  const handlers = {};
  const child = { on: (event, handler) => (handlers[event] = handler) };
  const io = { exitCode: undefined };
  launchNative(
    ["--timeout", "5", "dependencies", "src/a.ts"],
    (file, argv, options) => {
      calls.push({ file, argv, options });
      return child;
    },
    io,
  );
  handlers.exit(23, null);
  assert.deepEqual(calls[0].argv, ["--timeout", "5", "dependencies", "src/a.ts"]);
  assert.equal(io.exitCode, 23);
});

test("reports native spawn errors and preserves signal or null-code exits", () => {
  const handlers = {};
  const io = { exitCode: undefined, stderr: { write: (value) => (io.error = value) } };
  launchNative([], () => ({ on: (event, handler) => (handlers[event] = handler) }), io);
  handlers.error(new Error("spawn failed"));
  assert.equal(io.exitCode, 1);
  assert.equal(io.error, "spawn failed\n");
  handlers.exit(null, null);
  assert.equal(io.exitCode, 1);

  const signalHandlers = {};
  const signals = [];
  launchNative(
    [],
    () => ({ on: (event, handler) => (signalHandlers[event] = handler) }),
    { stderr: { write() {} } },
    (pid, signal) => signals.push({ pid, signal }),
  );
  signalHandlers.exit(null, "SIGTERM");
  assert.deepEqual(signals, [{ pid: process.pid, signal: "SIGTERM" }]);
});

test(
  "the real addon writes dependency, dependent, symbol, and plan artifacts",
  { skip: !process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH },
  async () => {
    const root = join(__dirname, "..", "..", "..", "fixtures", "napi", "real-addon-dependencies");
    const output = await mkdtemp(join(tmpdir(), "no-mistakes-planning-cli-"));
    const manifest = join(output, "changed-files.txt");
    try {
      await mkdir(output, { recursive: true, mode: 0o700 });
      await writeFile(manifest, "entry.ts\n");
      let stdout = "";
      await main(["--changed-files", manifest, "--output-dir", output], root, {
        stdout: { write: (value) => (stdout += value) },
      });
      assert.equal(stdout, "");
      for (const name of ["dependencies", "dependents", "symbols", "plan"]) {
        assert.equal(await readFile(join(output, `${name}.status`), "utf8"), "0\n");
        assert.notEqual(JSON.parse(await readFile(join(output, `${name}.json`), "utf8")), null);
      }
    } finally {
      await rm(output, { recursive: true, force: true });
    }
  },
);
