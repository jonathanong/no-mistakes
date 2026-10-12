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
  // Timeout/signal accounting can omit descendants the wrapper never waited for.
  // A completed nonzero check still has valid resources, despite being a failure.
  const resources = samples.filter(
    (sample) =>
      Number.isInteger(sample.exit) &&
      sample.exit >= 0 &&
      sample.exit !== 124 &&
      sample.exit !== 137 &&
      !sample.signal,
  );
  return {
    n: samples.length,
    passed: samples.filter(successful).length,
    timeouts: samples.filter((sample) => sample.exit === 124 || sample.exit === 137).length,
    max: sorted.at(-1),
    p95: sorted[Math.ceil(sorted.length * 0.95) - 1],
    resourceCount: resources.length,
    userMax: resources.length ? Math.max(...resources.map((sample) => sample.user)) : null,
    systemMax: resources.length ? Math.max(...resources.map((sample) => sample.system)) : null,
    rssMaxKiB: resources.length ? Math.max(...resources.map((sample) => sample.rss_kib)) : null,
  };
}

function formatSummary(rows) {
  return [
    "| Consumer commit / binary | n | Passed | Timeouts | Max s | p95 s | Resource n | Max user s | Max system s | Max RSS KiB |",
    "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ...rows.map(
      (row) =>
        `| ${row.key} | ${row.n} | ${row.passed} | ${row.timeouts} | ${row.max} | ${row.p95} | ${row.resourceCount} | ${row.userMax ?? "n/a"} | ${row.systemMax ?? "n/a"} | ${row.rssMaxKiB ?? "n/a"} |`,
    ),
    "",
    "p95 is the nearest-rank empirical percentile. With six fresh hosted pairs per tree it equals the observed maximum; this sample cannot guarantee the slow tail. Pairs share one runner and filesystem cache; binary order alternates. Timeout measurements include up to five seconds of termination cleanup and are failures, not completed checks. Resource maxima exclude exits 124/137, signals, and missing wrapper statuses because their descendant accounting may be incomplete; Resource n counts eligible samples, including completed nonzero exits. Raw measurements and deadline outcomes remain intact; n/a means no eligible resource samples. The npm baseline and current source build may use different compilers; inspect environment.json and provenance.json. Build, install, and tool preparation are outside the command deadline.",
  ].join("\n");
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
  formatSummary,
};
