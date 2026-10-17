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
  CHECK_ARGS,
  COLLECTION_DEADLINE_SECONDS,
  parseTime,
  successful,
  aggregate,
  summarize,
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
  const jobs = process.env.HOSTED_CHECK_JOBS || "auto";
  if (!["1", "4", "auto"].includes(jobs)) throw new Error(`Invalid jobs: ${jobs}`);
  const checkArgs = jobs === "auto" ? CHECK_ARGS : [...CHECK_ARGS, "--jobs", jobs];
  const metadata = {
    jobs,
    fixtureCommit,
    fixtureTree,
    configSha256: configBefore,
    command: ["pnpm", ...checkArgs],
    deadlineSeconds: COLLECTION_DEADLINE_SECONDS,
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
    rayonJobs:
      jobs !== "auto"
        ? Number(jobs)
        : process.env.RAYON_NUM_THREADS
          ? Number(process.env.RAYON_NUM_THREADS)
          : Number(command(join(artifact, "cpu-probe"), [], fixture)),
    rayonPolicy:
      jobs === "auto"
        ? "automatic (--jobs omitted; exact source-built num_cpus::get probe)"
        : "explicit --jobs",
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
            `${COLLECTION_DEADLINE_SECONDS}s`,
            "pnpm",
            ...checkArgs,
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
        jobs,
        candidateCommit: metadata.currentBuild.commit,
        order,
        sampleNumber,
        fixtureCommit,
        fixtureTree,
        binarySha256: hashFile(native),
        binaryVersion: command(native, ["--version"], fixture),
        signal: result.signal,
        processExit: result.status,
        deadlineSeconds: COLLECTION_DEADLINE_SECONDS,
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
    throw new Error(
      `Current check did not exit successfully within the ${COLLECTION_DEADLINE_SECONDS}-second collection cap`,
    );
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

module.exports = { CHECK_ARGS, COLLECTION_DEADLINE_SECONDS, parseTime, successful, aggregate };
