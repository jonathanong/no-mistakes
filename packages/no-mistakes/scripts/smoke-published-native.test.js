const assert = require("node:assert/strict");
const { chmodSync, mkdtempSync, mkdirSync, rmSync, writeFileSync } = require("node:fs");
const { tmpdir } = require("node:os");
const { join } = require("node:path");
const { spawnSync } = require("node:child_process");
const test = globalThis.test || require("node:test").test;

const {
  assertCliVersion,
  main,
  nativeCliPath,
  parseArgs,
  reportCliFailure,
  runIfMain,
  smokePublishedNative,
  smokePublishedRoot,
  startFromCli,
} = require("./smoke-published-native");

test("parses CLI flags and rejects incomplete argv", () => {
  assert.deepEqual(parseArgs(["--package", "pkg", "--version", "1.0.0"]), {
    package: "pkg",
    version: "1.0.0",
  });
  assert.throws(() => parseArgs(["--package"]), /--package requires a value/);
  assert.throws(() => parseArgs(["nope"]), /unexpected argument nope/);
});

test("prefers the Windows CLI filename when it exists", () => {
  assert.match(
    nativeCliPath("/pkg", () => true),
    /no-mistakes\.exe$/,
  );
  assert.match(
    nativeCliPath("/pkg", () => false),
    /no-mistakes$/,
  );
});

test("loads the published native addon and CLI", async () => {
  const cwd = mkdtempSync(join(tmpdir(), "smoke-native-"));
  try {
    const packageDir = join(cwd, "node_modules", "fake-native");
    mkdirSync(join(packageDir, "bin"), { recursive: true });
    writeFileSync(
      join(packageDir, "package.json"),
      JSON.stringify({ name: "fake-native", version: "1.2.3", main: "index.js" }),
    );
    writeFileSync(
      join(packageDir, "index.js"),
      "module.exports = { version: async () => '1.2.3' };\n",
    );
    const cli = join(packageDir, "bin", "no-mistakes");
    writeFileSync(cli, "#!/usr/bin/env node\nprocess.stdout.write('no-mistakes 1.2.3\\n');\n");
    chmodSync(cli, 0o755);
    const extras =
      process.platform === "win32"
        ? { spawn: () => ({ status: 0, stdout: "no-mistakes 1.2.3\n", stderr: "" }) }
        : {};
    const result = await smokePublishedNative({
      name: "fake-native",
      version: "1.2.3",
      cwd,
      ...extras,
    });
    assert.equal(result.cli, cli);
    assert.match(result.output, /1\.2\.3/);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

test("fails when the native addon, CLI, or version output is wrong", async () => {
  await assert.rejects(smokePublishedNative({}), /name and version are required/);
  await assert.rejects(
    smokePublishedNative({
      name: "pkg",
      version: "1.0.0",
      load: () => ({ version: async () => "9.9.9" }),
    }),
    /native addon version mismatch/,
  );
  await assert.rejects(
    smokePublishedNative({
      name: "pkg",
      version: "1.0.0",
      cwd: "/missing",
      load: () => ({ version: async () => "1.0.0" }),
      exists: () => false,
    }),
    /missing CLI/,
  );
  await assert.rejects(
    smokePublishedNative({
      name: "pkg",
      version: "1.0.0",
      load: () => ({ version: async () => "1.0.0" }),
      exists: () => true,
      spawn: () => ({ status: 1, stdout: "", stderr: "spawn failed\n" }),
    }),
    /spawn failed/,
  );
  assert.throws(
    () => assertCliVersion({ status: 0, stdout: "other", stderr: "" }, "1.0.0", "cli"),
    /missing 1.0.0/,
  );
  assert.throws(
    () => assertCliVersion({ status: 2, stdout: "", stderr: "" }, "1.0.0", "cli"),
    /cli --version failed/,
  );
});

test("loads the published root package launcher", async () => {
  const cwd = mkdtempSync(join(tmpdir(), "smoke-root-"));
  try {
    const packageDir = join(cwd, "node_modules", "no-mistakes");
    mkdirSync(join(packageDir, "bin"), { recursive: true });
    writeFileSync(
      join(packageDir, "package.json"),
      JSON.stringify({ name: "no-mistakes", version: "4.5.6", main: "index.js" }),
    );
    writeFileSync(join(packageDir, "index.js"), "module.exports = { ok: true };\n");
    writeFileSync(
      join(packageDir, "bin", "no-mistakes.js"),
      "process.stdout.write('no-mistakes 4.5.6\\n');\n",
    );
    const result = await smokePublishedRoot({
      version: "4.5.6",
      cwd,
    });
    assert.match(result.launcher, /no-mistakes\.js$/);
    assert.match(result.output, /4\.5\.6/);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

test("root smoke requires a version", async () => {
  await assert.rejects(smokePublishedRoot({}), /version is required/);
});

test("root smoke loads the installed package, not the repository package", () => {
  const cwd = mkdtempSync(join(tmpdir(), "smoke-root-isolation-"));
  try {
    const packageDir = join(cwd, "node_modules", "no-mistakes");
    mkdirSync(join(packageDir, "bin"), { recursive: true });
    writeFileSync(
      join(packageDir, "package.json"),
      JSON.stringify({ name: "no-mistakes", version: "9.9.9", main: "index.js" }),
    );
    writeFileSync(join(packageDir, "index.js"), "module.exports = { ok: true };\n");
    writeFileSync(
      join(packageDir, "bin", "no-mistakes.js"),
      "process.stdout.write('no-mistakes 9.9.9\\n');\n",
    );
    const result = spawnSync(
      process.execPath,
      [
        require.resolve("./smoke-published-native.js"),
        "--package",
        "no-mistakes",
        "--version",
        "9.9.9",
      ],
      { cwd, encoding: "utf8" },
    );
    assert.equal(result.status, 0, result.stderr);
    assert.match(result.stdout, /no-mistakes\.js/);
    assert.doesNotMatch(result.stderr, /no staged artifacts/);
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
});

test("main dispatches root vs native packages", async () => {
  const chunks = [];
  await main(["--package", "no-mistakes", "--version", "0.1.0"], {
    stdout: { write: (chunk) => chunks.push(chunk) },
    load: () => ({}),
    spawn: () => ({ status: 0, stdout: "0.1.0", stderr: "" }),
  });
  assert.match(chunks.join(""), /no-mistakes\.js/);
  chunks.length = 0;
  await main(["--package", "no-mistakes-linux-x64-gnu", "--version", "0.1.0"], {
    stdout: { write: (chunk) => chunks.push(chunk) },
    load: () => ({ version: async () => "0.1.0" }),
    exists: () => true,
    spawn: () => ({ status: 0, stdout: "0.1.0", stderr: "" }),
  });
  assert.match(chunks.join(""), /no-mistakes-linux-x64-gnu/);
  const writes = [];
  const originalWrite = process.stdout.write.bind(process.stdout);
  process.stdout.write = (chunk) => {
    writes.push(String(chunk));
    return true;
  };
  try {
    await main(["--package", "no-mistakes", "--version", "0.1.0"], {
      load: () => ({}),
      spawn: () => ({ status: 0, stdout: "0.1.0", stderr: "" }),
    });
  } finally {
    process.stdout.write = originalWrite;
  }
  assert.match(writes.join(""), /no-mistakes\.js/);
});

test("CLI reports usage errors without installing", () => {
  const result = spawnSync(
    process.execPath,
    [require.resolve("./smoke-published-native.js"), "--package"],
    { encoding: "utf8" },
  );
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /--package requires a value/);
});

test("runs the smoke checker only when the module is executed directly", async () => {
  let started = false;
  runIfMain(module, module, () => {
    started = true;
  });
  assert.equal(started, true);
  runIfMain({}, module, () => {
    started = false;
  });
  assert.equal(started, true);
  await startFromCli(async () => {}, () => {});
  let caught;
  await startFromCli(
    async () => {
      throw new Error("cli failed");
    },
    (error) => {
      caught = error;
    },
  );
  assert.equal(caught.message, "cli failed");
});

test("reportCliFailure writes the error and sets a nonzero exit", () => {
  const chunks = [];
  const io = { stderr: { write: (chunk) => chunks.push(chunk) }, exitCode: 0 };
  reportCliFailure(new Error("boom"), io);
  assert.equal(io.exitCode, 1);
  assert.deepEqual(chunks, ["boom\n"]);
});
