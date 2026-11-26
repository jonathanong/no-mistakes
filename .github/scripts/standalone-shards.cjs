const { spawnSync } = require("node:child_process");
const { existsSync, mkdirSync, readFileSync, writeFileSync } = require("node:fs");
const path = require("node:path");

const SHARDS = 3;

function archiveName(shard) {
  return `standalone-${shard}.tar.zst`;
}

function commands(mode, bundle, index, total, workspace = process.cwd()) {
  if (mode === "build") {
    if (index === undefined)
      return Array.from({ length: SHARDS }, (_, shard) => buildCommand(bundle, shard));
    if (!Number.isInteger(index) || index < 1 || index > SHARDS)
      throw new Error("expected a one-based standalone build shard");
    return [buildCommand(bundle, index - 1)];
  }
  if (!Number.isInteger(index) || !Number.isInteger(total) || index < 1 || index > total)
    throw new Error("expected a one-based standalone shard within its total");
  if (total !== SHARDS) throw new Error("standalone shard total must match the compiled archives");
  return [
    [
      "nextest",
      "run",
      "--archive-file",
      path.resolve(bundle, archiveName(index - 1)),
      "--workspace-remap",
      workspace,
      "--extract-to",
      path.resolve("target/standalone-lib"),
      "--no-fail-fast",
      "--retries",
      "0",
    ],
  ];
}

function buildCommand(bundle, shard) {
  return [
    "nextest",
    "archive",
    "--locked",
    ...(shard === 0 ? ["--workspace"] : ["-p", "no-mistakes"]),
    "--lib",
    "--archive-file",
    path.resolve(bundle, archiveName(shard)),
    "--zstd-level",
    "6",
    "--zstd-long",
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
    delete childEnv.NM_TEST_SHARD;
  }
  for (const [offset, args] of commands(mode, directory, index, total).entries()) {
    const shardEnv = { ...childEnv };
    if (mode === "build") {
      if (shardEnv.NM_TEST_SHARD) throw new Error("standalone compile shard is already selected");
      const shard = index === undefined ? offset : index - 1;
      shardEnv.NM_TEST_SHARD = String(shard);
      const wrapper = path.join(__dirname, "standalone-shard-rustc.sh");
      // Cargo fingerprints workspace artifacts by the wrapper path.
      shardEnv.RUSTC_WORKSPACE_WRAPPER = path.join(__dirname, `standalone-shard-rustc-${shard}.sh`);
      if (shardEnv.RUSTC_WRAPPER && shardEnv.RUSTC_WRAPPER !== wrapper) {
        shardEnv.NM_INNER_RUSTC_WRAPPER = shardEnv.RUSTC_WRAPPER;
      }
      shardEnv.RUSTC_WRAPPER = "";
    }
    const child = spawnSync("cargo", args, { env: shardEnv, stdio: "inherit" });
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
  if (mode === "build" && (index === undefined || index === SHARDS)) {
    for (let shard = 0; shard < SHARDS; shard += 1) {
      if (!existsSync(path.join(directory, archiveName(shard))))
        throw new Error("standalone build requires all three archives before publication");
    }
    writeFileSync(
      manifestPath,
      `${JSON.stringify({ sha: env.GITHUB_SHA, workspace: process.cwd() })}\n`,
    );
  }
  return 0;
}

if (require.main === module) {
  try {
    const [mode, bundle, index, total] = process.argv.slice(2);
    const arity = process.argv.length - 2;
    if (!bundle || (mode === "build" ? ![2, 3].includes(arity) : arity !== 4))
      throw new Error(
        "usage: standalone-shards.cjs build <bundle> [index] | run <bundle> <index> <total>",
      );
    process.exitCode = run(
      mode,
      bundle,
      index === undefined ? undefined : Number(index),
      Number(total),
    );
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
module.exports = { commands, run, SHARDS };
