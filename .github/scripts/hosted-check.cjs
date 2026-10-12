const { spawnSync } = require("node:child_process");
const {
  copyFileSync,
  readFileSync,
  writeFileSync,
  mkdirSync,
  openSync,
  closeSync,
} = require("node:fs");
const { availableParallelism, cpus } = require("node:os");
const { createRequire } = require("node:module");
const { join, resolve, dirname } = require("node:path");

const {
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
} = require("./hosted-check-metrics.cjs");

function measure(fixture, artifact, output, sampleNumber, expectedTree) {
  mkdirSync(output, { recursive: true });
  const fixtureCommit = command("git", ["rev-parse", "HEAD"], fixture);
  const fixtureTree = command("git", ["rev-parse", "HEAD^{tree}"], fixture);
  if (fixtureTree !== expectedTree) throw new Error(`Unexpected consumer tree: ${fixtureTree}`);
  const rootPackage = packageManifest(
    createRequire(join(fixture, "package.json")).resolve,
    "no-mistakes",
  );
  const nativePackage = require.resolve("no-mistakes-linux-x64-gnu/package.json", {
    paths: [dirname(rootPackage)],
  });
  for (const path of [rootPackage, nativePackage]) {
    if (JSON.parse(readFileSync(path, "utf8")).version !== "0.86.2")
      throw new Error(`Expected published 0.86.2: ${path}`);
  }
  const native = join(dirname(nativePackage), "bin", "no-mistakes");
  const baseline = join(output, "published-no-mistakes");
  copyFileSync(native, baseline);
  const configBefore = hashFile(join(fixture, ".no-mistakes.yml"));
  const metadata = {
    fixtureCommit,
    fixtureTree,
    configSha256: configBefore,
    command: ["pnpm", ...CHECK_ARGS],
    deadlineSeconds: DEADLINE_SECONDS,
    sampleNumber,
    imageOS: process.env.ImageOS,
    imageVersion: process.env.ImageVersion,
    runnerOS: process.env.RUNNER_OS,
    runnerArch: process.env.RUNNER_ARCH,
    cpuModel: cpus()[0].model,
    logicalCpus: cpus().length,
    availableCpus: availableParallelism(),
    nproc: command("nproc", [], fixture),
    rayonEnvironment: process.env.RAYON_NUM_THREADS || null,
    numCpus: Number(command(join(artifact, "cpu-probe"), [], fixture)),
    rayonJobs: process.env.RAYON_NUM_THREADS
      ? Number(process.env.RAYON_NUM_THREADS)
      : Number(command(join(artifact, "cpu-probe"), [], fixture)),
    rayonPolicy: "automatic (--jobs omitted; exact source-built num_cpus::get probe)",
    node: process.version,
    pnpm: command("pnpm", ["--version"], fixture),
    baselineNpm: JSON.parse(
      command("pnpm", ["view", "no-mistakes-linux-x64-gnu@0.86.2", "--json"], fixture),
    ),
    currentBuild: JSON.parse(readFileSync(join(artifact, "provenance.json"), "utf8")),
    wrapper:
      "Published npm no-mistakes@0.86.2 launcher for both binaries; only its untracked native CLI is replaced",
  };
  writeFileSync(join(output, "environment.json"), JSON.stringify(metadata, null, 2));
  writeFileSync(join(output, "cpu.txt"), command("lscpu", [], fixture));
  const variants = sampleNumber % 2 ? ["published", "current"] : ["current", "published"];
  let failed = false;
  try {
    for (const [order, variant] of variants.entries()) {
      copyFileSync(variant === "published" ? baseline : join(artifact, "no-mistakes"), native);
      const prefix = join(output, variant);
      const stdout = openSync(`${prefix}.stdout`, "w");
      const stderr = openSync(`${prefix}.stderr`, "w");
      let result;
      try {
        result = spawnSync(
          "/usr/bin/time",
          [
            "-o",
            `${prefix}.time`,
            "-f",
            "elapsed=%e\nuser=%U\nsystem=%S\nrss_kib=%M\nexit=%x",
            "timeout",
            "--signal=TERM",
            "--kill-after=5s",
            `${DEADLINE_SECONDS}s`,
            "pnpm",
            ...CHECK_ARGS,
          ],
          { cwd: fixture, stdio: ["ignore", stdout, stderr] },
        );
      } finally {
        closeSync(stdout);
        closeSync(stderr);
      }
      if (result.error) throw result.error;
      const sample = {
        ...parseTime(readFileSync(`${prefix}.time`, "utf8"), result.status),
        variant,
        order,
        sampleNumber,
        fixtureCommit,
        fixtureTree,
        binarySha256: hashFile(native),
        binaryVersion: command(native, ["--version"], fixture),
        signal: result.signal,
        processExit: result.status,
        deadlineSeconds: DEADLINE_SECONDS,
      };
      writeFileSync(`${prefix}.sample.json`, JSON.stringify(sample, null, 2));
      console.log(JSON.stringify(sample));
      if (variant === "current" && !successful(sample)) failed = true;
    }
  } finally {
    copyFileSync(baseline, native);
  }
  if (hashFile(join(fixture, ".no-mistakes.yml")) !== configBefore)
    throw new Error("Consumer config changed");
  command("git", ["diff", "--exit-code", "HEAD"], fixture);
  if (failed)
    throw new Error("Current check did not exit successfully before the 180-second deadline");
}

function summarize(directory) {
  const samples = sampleFiles(directory).map((path) => JSON.parse(readFileSync(path, "utf8")));
  const groups = new Map();
  for (const sample of samples) {
    const key = `${sample.fixtureCommit}/${sample.variant}`;
    if (!groups.has(key)) groups.set(key, []);
    groups.get(key).push(sample);
  }
  const rows = [...groups].sort().map(([key, values]) => ({ key, ...aggregate(values) }));
  const table = formatSummary(rows);
  writeFileSync(join(directory, "summary.json"), JSON.stringify(rows, null, 2));
  writeFileSync(join(directory, "summary.md"), table);
  if (process.env.GITHUB_STEP_SUMMARY) writeFileSync(process.env.GITHUB_STEP_SUMMARY, table);
  console.log(table);
  const expected = [
    "22edf4bfe0a782ac0d030c916dedfd6f8acb499c",
    "92d0a2061bde8d260aa0b86a2d71110ece1a43ca",
  ];
  for (const commit of expected) {
    for (const variant of ["published", "current"]) {
      const values = groups.get(`${commit}/${variant}`) || [];
      if (values.length !== 6 || new Set(values.map((sample) => sample.sampleNumber)).size !== 6)
        throw new Error(`Incomplete samples: ${commit}/${variant}`);
      if (variant === "current" && !values.every(successful))
        throw new Error(`Current check failed: ${commit}`);
    }
  }
}

if (require.main === module) {
  try {
    if (process.argv[2] === "measure")
      measure(
        resolve(process.argv[3]),
        resolve(process.argv[4]),
        resolve(process.argv[5]),
        Number(process.argv[6]),
        process.argv[7],
      );
    else if (process.argv[2] === "summarize") summarize(resolve(process.argv[3]));
    else
      throw new Error(
        "Usage: hosted-check.cjs measure FIXTURE ARTIFACT OUTPUT SAMPLE TREE | summarize DIRECTORY",
      );
  } catch (error) {
    console.error(error);
    process.exitCode = 1;
  }
}

module.exports = { CHECK_ARGS, DEADLINE_SECONDS, parseTime, successful, aggregate };
