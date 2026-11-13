const { spawnSync } = require("node:child_process");
const { existsSync, mkdirSync, readFileSync, writeFileSync } = require("node:fs");
const path = require("node:path");

function commands(mode, bundle, index, total, workspace = process.cwd()) {
  const archive = path.resolve(bundle, "standalone.tar.zst");
  if (mode === "build")
    return [
      ["nextest", "archive", "--locked", "-p", "no-mistakes", "--lib", "--archive-file", archive],
    ];
  if (!Number.isInteger(index) || !Number.isInteger(total) || index < 1 || index > total)
    throw new Error("expected a one-based standalone shard within its total");
  return [
    [
      "nextest",
      "run",
      "--archive-file",
      archive,
      "--workspace-remap",
      workspace,
      "--extract-to",
      path.resolve("target/standalone-lib"),
      "--partition",
      `hash:${index}/${total}`,
      "--no-fail-fast",
      "--retries",
      "0",
    ],
  ];
}

function run(mode, bundle, index, total, env = process.env) {
  const directory = path.resolve(bundle);
  const manifestPath = path.join(directory, "build.json");
  if (!["build", "run"].includes(mode)) throw new Error("expected build or run");
  if (!env.GITHUB_SHA) throw new Error("standalone archive requires source revision");
  const childEnv = { ...env };
  delete childEnv["CARGO_BIN_EXE_no-mistakes"];
  if (mode === "build") mkdirSync(directory, { recursive: true });
  else {
    const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
    if (manifest.sha !== env.GITHUB_SHA || manifest.workspace !== process.cwd())
      throw new Error("standalone archive source or workspace differs");
    mkdirSync(path.resolve("target/standalone-lib"), { recursive: true });
    childEnv.CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER = `bash ${path.join(__dirname, "libtest-argv-runner.sh")}`;
  }
  for (const args of commands(mode, directory, Number(index), Number(total))) {
    const child = spawnSync("cargo", args, { env: childEnv, stdio: "inherit" });
    if (child.error) throw child.error;
    if (child.status !== 0) return child.status ?? 1;
  }
  const target =
    mode === "build" ? env.CARGO_TARGET_DIR : path.resolve("target/standalone-lib/target");
  if (!target) throw new Error("standalone archive requires its isolated Cargo target");
  if (
    ["no-mistakes", "no-mistakes.exe"].some((name) => existsSync(path.join(target, "debug", name)))
  )
    throw new Error("standalone library validation must not build the CLI");
  if (mode === "build")
    writeFileSync(
      manifestPath,
      `${JSON.stringify({ sha: env.GITHUB_SHA, workspace: process.cwd() })}\n`,
    );
  return 0;
}

if (require.main === module) {
  try {
    const [mode, bundle, index, total] = process.argv.slice(2);
    if (!bundle || (mode === "build" ? process.argv.length !== 4 : process.argv.length !== 6))
      throw new Error("usage: standalone-shards.cjs build <bundle> | run <bundle> <index> <total>");
    process.exitCode = run(mode, bundle, index, total);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
module.exports = { commands, run };
