const { spawnSync } = require("node:child_process");
const {
  copyFileSync,
  readFileSync,
  writeFileSync,
  mkdirSync,
  openSync,
  closeSync,
} = require("node:fs");
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
const { replaceNative, measurementEnvironment } = require("./hosted-check-runtime.cjs");
const { artifactBuilds } = require("./hosted-check-provenance.cjs");

function measure(fixture, artifact, output, sampleNumber, expectedTree) {
  if (!Number.isInteger(sampleNumber) || sampleNumber < 1 || sampleNumber > 6)
    throw new Error("Expected sample number 1 through 6");
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
  const builds = artifactBuilds(artifact);
  const native = join(dirname(nativePackage), "bin", "no-mistakes");
  const original = join(output, "launcher-original-no-mistakes");
  copyFileSync(native, original);
  const cpuProbes = Object.fromEntries(
    ["baseline", "candidate"].map((variant) => [
      variant,
      Number(command(join(artifact, variant, "cpu-probe"), [], fixture)),
    ]),
  );
  if (!Object.values(cpuProbes).every((count) => Number.isInteger(count) && count > 0))
    throw new Error("Invalid source-built CPU probe result");
  const configBefore = hashFile(join(fixture, ".no-mistakes.yml"));
  const jobs = process.env.HOSTED_CHECK_JOBS || "auto";
  if (!["1", "4", "auto"].includes(jobs)) throw new Error(`Invalid jobs: ${jobs}`);
  const checkArgs = jobs === "auto" ? CHECK_ARGS : [...CHECK_ARGS, "--jobs", jobs];
  const metadata = measurementEnvironment(fixture, rootPackage, checkArgs, {
    jobs,
    fixtureCommit,
    fixtureTree,
    configSha256: configBefore,
    sampleNumber,
    cpuProbes,
    builds,
  });
  writeFileSync(join(output, "environment.json"), JSON.stringify(metadata, null, 2));
  writeFileSync(join(output, "cpu.txt"), command("lscpu", [], fixture));
  const variants = sampleNumber % 2 ? ["baseline", "candidate"] : ["candidate", "baseline"];
  let failed = false;
  try {
    for (const [order, variant] of variants.entries()) {
      replaceNative(join(artifact, variant, "no-mistakes"), native);
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
        candidateCommit: builds.candidate.commit,
        sourceCommit: builds[variant].commit,
        sourceTree: builds[variant].tree,
        provenance: builds[variant],
        numCpus: cpuProbes[variant],
        rayonJobs:
          jobs === "auto"
            ? Number(process.env.RAYON_NUM_THREADS || cpuProbes[variant])
            : Number(jobs),
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
      if (variant === "candidate" && !successful(sample)) failed = true;
    }
  } finally {
    replaceNative(original, native);
  }
  if (hashFile(join(fixture, ".no-mistakes.yml")) !== configBefore)
    throw new Error("Consumer config changed");
  command("git", ["diff", "--exit-code", "HEAD"], fixture);
  if (failed)
    throw new Error(
      `Candidate check did not exit successfully within the ${COLLECTION_DEADLINE_SECONDS}-second collection cap`,
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
