const { spawnSync } = require("node:child_process");
const { createHash } = require("node:crypto");
const { readFileSync, copyFileSync, renameSync, rmSync } = require("node:fs");
const { availableParallelism, cpus } = require("node:os");
const { dirname, join } = require("node:path");

const CHECK_ARGS = ["exec", "no-mistakes", "--timeout", "0", "--lock-timeout", "0", "check"];
// Collection safety cap; regression limits come from reviewed measured budgets.
const COLLECTION_DEADLINE_SECONDS = 600;
// Stay in the caller's process group so SIGTERM reaches pnpm, which relays to
// `pnpm exec` (no TTY). GNU timeout's default group does not include that child.
// `--foreground` does not time out descendants; leftover analyzers are killed
// separately after each sample because the JS launcher does not forward signals.
const TIMEOUT_ARGS = ["--foreground", "--signal=TERM", "--kill-after=5s"];
const LEFTOVER_KILL_GRACE_MS = 5000;

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
function resourceAccountingComplete(sample) {
  return (
    Number.isInteger(sample.exit) &&
    sample.exit >= 0 &&
    sample.exit !== 124 &&
    sample.exit !== 137 &&
    !sample.signal
  );
}

function reportedSample(sample) {
  if (resourceAccountingComplete(sample)) return sample;
  // A killed launcher may not reap its analyzer: tiny wrapper counters mislead.
  // Preserve the original GNU time record separately in the .time artifact.
  return {
    ...sample,
    user: null,
    system: null,
    rss_kib: null,
    resourceStatus:
      sample.exit === 124 || sample.exit === 137 || sample.signal
        ? "n/a (killed)"
        : "n/a (missing wrapper status)",
  };
}
function command(executable, args, cwd) {
  const result = spawnSync(executable, args, { cwd, encoding: "utf8" });
  if (result.error || result.status !== 0) {
    throw new Error(`${executable} ${args.join(" ")}: ${result.error || result.stderr}`);
  }
  return result.stdout.trim();
}

function leftoverCheckPids(psOutput, nativePath, selfPid = process.pid) {
  if (typeof nativePath !== "string" || nativePath.length === 0) {
    throw new Error("nativePath is required");
  }
  const pids = [];
  for (const line of psOutput.split("\n")) {
    const match = line.trim().match(/^(\d+)\s+(\S+)\s+(.*)$/);
    if (!match) continue;
    const pid = Number(match[1]);
    const comm = match[2];
    const args = match[3];
    if (!Number.isInteger(pid) || pid <= 1 || pid === selfPid) continue;
    if (args.includes("hosted-check.cjs")) continue;
    const tokens = args.split(/\s+/);
    const native =
      comm === "no-mistakes" ||
      tokens.includes(nativePath) ||
      tokens.some((token) => token.endsWith("/bin/no-mistakes.js") || token === "no-mistakes.js");
    if (native) pids.push(pid);
  }
  return pids;
}

function sleepMs(ms) {
  spawnSync("sleep", [String(ms / 1000)], { encoding: "utf8" });
}

function terminateLeftoverCheckProcesses({
  psOutput,
  nativePath,
  kill = (pid, signal) => process.kill(pid, signal),
  sleep = sleepMs,
  selfPid = process.pid,
  graceMs = LEFTOVER_KILL_GRACE_MS,
} = {}) {
  const pids = leftoverCheckPids(psOutput, nativePath, selfPid);
  for (const pid of pids) {
    try {
      kill(pid, "SIGTERM");
    } catch {
      // Already gone or not owned by this runner.
    }
  }
  if (pids.length > 0) sleep(graceMs);
  for (const pid of pids) {
    try {
      kill(pid, "SIGKILL");
    } catch {
      // Already gone or not owned by this runner.
    }
  }
  return pids;
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
    rayonJobs:
      jobs === "auto" ? Number(process.env.RAYON_NUM_THREADS || cpuProbes.candidate) : Number(jobs),
    rayonPolicy:
      jobs === "auto"
        ? "automatic (--jobs omitted; exact source-built num_cpus::get probe)"
        : "explicit --jobs",
    node: process.version,
    pnpm: command("pnpm", ["--version"], fixture),
    launcher: JSON.parse(readFileSync(rootPackage, "utf8")).version,
    wrapper:
      "One installed npm launcher for both source-built binaries; only its untracked native CLI is replaced",
  };
}

module.exports = {
  measurementEnvironment,
  replaceNative,
  command,
  hashFile,
  packageManifest,
  leftoverCheckPids,
  terminateLeftoverCheckProcesses,
  CHECK_ARGS,
  TIMEOUT_ARGS,
  LEFTOVER_KILL_GRACE_MS,
  COLLECTION_DEADLINE_SECONDS,
  successful,
  resourceAccountingComplete,
  reportedSample,
};
