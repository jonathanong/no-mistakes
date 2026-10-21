const { spawnSync } = require("node:child_process");
const { createHash } = require("node:crypto");
const { readFileSync, copyFileSync, renameSync, rmSync } = require("node:fs");
const { availableParallelism, cpus } = require("node:os");
const { dirname, join } = require("node:path");

const CHECK_ARGS = ["exec", "no-mistakes", "--timeout", "0", "--lock-timeout", "0", "check"];
// Collection safety cap; regression limits come from reviewed measured budgets.
const COLLECTION_DEADLINE_SECONDS = 600;

function successful(sample) {
  const deadline = sample.deadlineSeconds ?? COLLECTION_DEADLINE_SECONDS;
  return (
    sample.exit === 0 &&
    Number.isFinite(deadline) &&
    deadline > 0 &&
    sample.elapsed < deadline &&
    !sample.signal
  );
}
function command(executable, args, cwd) {
  const result = spawnSync(executable, args, { cwd, encoding: "utf8" });
  if (result.error || result.status !== 0) {
    throw new Error(`${executable} ${args.join(" ")}: ${result.error || result.stderr}`);
  }
  return result.stdout.trim();
}

function hashFile(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function packageManifest(resolvePackage, name) {
  // The public package exports its entrypoint, but keeps package.json private.
  return join(dirname(resolvePackage(name)), "package.json");
}

function replaceNative(source, destination) {
  // Rename replaces the directory entry without modifying a pnpm store hardlink.
  const temporary = `${destination}.${process.pid}.swap`;
  try {
    copyFileSync(source, temporary);
    renameSync(temporary, destination);
  } finally {
    rmSync(temporary, { force: true });
  }
}

function measurementEnvironment(fixture, rootPackage, checkArgs, details) {
  const { jobs, cpuProbes } = details;
  return {
    ...details,
    command: ["pnpm", ...checkArgs],
    deadlineSeconds: COLLECTION_DEADLINE_SECONDS,
    imageOS: process.env.ImageOS,
    imageVersion: process.env.ImageVersion,
    runnerOS: process.env.RUNNER_OS,
    runnerArch: process.env.RUNNER_ARCH,
    cpuModel: cpus()[0].model,
    logicalCpus: cpus().length,
    availableCpus: availableParallelism(),
    nproc: command("nproc", [], fixture),
    rayonEnvironment: process.env.RAYON_NUM_THREADS || null,
    numCpus: cpuProbes.candidate,
    rayonJobs: jobs === "auto" ? Number(process.env.RAYON_NUM_THREADS || cpuProbes.candidate) : Number(jobs),
    rayonPolicy: jobs === "auto"
      ? "automatic (--jobs omitted; exact source-built num_cpus::get probe)" : "explicit --jobs",
    node: process.version,
    pnpm: command("pnpm", ["--version"], fixture),
    launcher: JSON.parse(readFileSync(rootPackage, "utf8")).version,
    wrapper: "One installed npm launcher for both source-built binaries; only its untracked native CLI is replaced",
  };
}

module.exports = {
  measurementEnvironment,
  replaceNative,
  command,
  hashFile,
  packageManifest,
  CHECK_ARGS,
  COLLECTION_DEADLINE_SECONDS,
  successful,
};
