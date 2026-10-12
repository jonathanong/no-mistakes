const { spawnSync } = require("node:child_process");
const { createHash } = require("node:crypto");
const { readFileSync, readdirSync } = require("node:fs");
const { dirname, join } = require("node:path");
const CHECK_ARGS = ["exec", "no-mistakes", "--timeout", "0", "--lock-timeout", "0", "check"];
const DEADLINE_SECONDS = 180;

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

function parseTime(text, processExit) {
  const values = Object.fromEntries(
    text
      .split("\n")
      .filter((line) => /^[a-z_]+=/.test(line))
      .map((line) => line.split("=")),
  );
  const result = {};
  for (const key of ["elapsed", "user", "system", "rss_kib", "exit"]) {
    if (!values[key] || !Number.isFinite(Number(values[key])))
      throw new Error(`Missing GNU time field: ${key}`);
    result[key] = Number(values[key]);
  }
  // GNU time's %x can be zero for a command killed by a signal. Preserve it,
  // but use the actual wrapper exit so SIGKILL never becomes a successful check.
  if (processExit !== undefined) {
    result.timeExit = result.exit;
    result.exit = processExit;
  }
  return result;
}

function successful(sample) {
  return sample.exit === 0 && sample.elapsed < DEADLINE_SECONDS && !sample.signal;
}

function aggregate(samples) {
  if (!samples.length) throw new Error("Cannot summarize an empty sample");
  const sorted = samples.map((sample) => sample.elapsed).sort((a, b) => a - b);
  return {
    n: samples.length,
    passed: samples.filter(successful).length,
    timeouts: samples.filter((sample) => sample.exit === 124 || sample.exit === 137).length,
    max: sorted.at(-1),
    p95: sorted[Math.ceil(sorted.length * 0.95) - 1],
    userMax: Math.max(...samples.map((sample) => sample.user)),
    systemMax: Math.max(...samples.map((sample) => sample.system)),
    rssMaxKiB: Math.max(...samples.map((sample) => sample.rss_kib)),
  };
}

function sampleFiles(directory) {
  return readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const path = join(directory, entry.name);
    return entry.isDirectory()
      ? sampleFiles(path)
      : entry.name.endsWith(".sample.json")
        ? [path]
        : [];
  });
}

module.exports = {
  command,
  hashFile,
  packageManifest,
  sampleFiles,
  CHECK_ARGS,
  DEADLINE_SECONDS,
  parseTime,
  successful,
  aggregate,
};
