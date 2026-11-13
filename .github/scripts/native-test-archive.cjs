const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");
const { targets } = require("./native-rust-tests.cjs");
const {
  relativeExecutable,
  digest,
  executionPlan,
  parseListing,
  casePlan,
  batchPlan,
} = require("./native-test-artifacts.cjs");
const bundle = "target/native-test-artifacts";

function completed(result) {
  if (result.error) throw result.error;
  if (result.status !== 0) {
    const error = new Error(`native process failed: ${result.signal || result.status}`);
    error.status = result.status ?? 1;
    throw error;
  }
}

function build() {
  if (process.env.NO_MISTAKES_BUILD_NAPI !== "1")
    throw new Error("native Rust tests require NO_MISTAKES_BUILD_NAPI=1");
  fs.rmSync(bundle, { recursive: true, force: true });
  const files = new Map();
  let cli;
  const entries = targets(process.env.RUNNER_OS).map(({ selector }, index) => {
    completed(
      spawnSync(
        process.execPath,
        [path.join(__dirname, "build-native.cjs"), "--archive-tests", String(index)],
        { stdio: "inherit" },
      ),
    );
    const artifacts = JSON.parse(
      fs.readFileSync(`target/native-test-build-diagnostics/${index}/artifacts.json`, "utf8"),
    );
    for (const artifact of artifacts) {
      const relative = relativeExecutable(process.cwd(), artifact.executable);
      const output = path.join(bundle, relative);
      fs.mkdirSync(path.dirname(output), { recursive: true });
      fs.copyFileSync(artifact.executable, output);
      files.set(relative, { path: relative, sha256: digest(output) });
      if (!artifact.test && artifact.name === "no-mistakes" && artifact.kind.includes("bin"))
        cli = relative;
    }
    const matches = artifacts.filter(
      (artifact) =>
        artifact.test &&
        artifact.name === selector[1] &&
        artifact.kind.includes(selector[0].slice(2)),
    );
    if (matches.length !== 1)
      throw new Error(`expected one native harness for ${selector.join(" ")}`);
    return {
      name: selector[1],
      kind: selector[0].slice(2),
      executable: relativeExecutable(process.cwd(), matches[0].executable),
    };
  });
  const inventory = executionPlan(process.env.RUNNER_OS, entries).map((group) => {
    const result = spawnSync(
      path.resolve(group.executable),
      [...group.args, "--list", "--format", "terse"],
      {
        cwd: path.resolve("crates/no-mistakes"),
        encoding: "utf8",
        maxBuffer: 16 * 1024 * 1024,
      },
    );
    completed(result);
    return { ...group, tests: parseListing(result.stdout) };
  });
  casePlan(process.env.RUNNER_OS, entries, inventory, 1, 1);
  fs.writeFileSync(path.join(bundle, "inventory.json"), `${JSON.stringify(inventory, null, 2)}\n`);
  if (!cli) throw new Error("native archive is missing the CLI used by integration tests");
  fs.writeFileSync(
    path.join(bundle, "manifest.json"),
    `${JSON.stringify(
      {
        workspace: process.cwd(),
        platform: process.env.RUNNER_OS,
        sha: process.env.GITHUB_SHA,
        cli,
        inventorySha256: digest(path.join(bundle, "inventory.json")),
        entries,
        files: [...files.values()],
      },
      null,
      2,
    )}\n`,
  );
}

function restore(manifest, directory = bundle) {
  if (manifest.workspace !== process.cwd())
    throw new Error(
      "native archive requires the producer checkout path (embedded fixture and CLI paths)",
    );
  if (manifest.platform !== process.env.RUNNER_OS || manifest.sha !== process.env.GITHUB_SHA)
    throw new Error("native archive platform or source SHA mismatch");
  for (const file of manifest.files) {
    const destination = path.resolve(file.path);
    if (relativeExecutable(process.cwd(), destination) !== file.path)
      throw new Error(`invalid native archive path: ${file.path}`);
    const source = path.join(directory, file.path);
    if (digest(source) !== file.sha256)
      throw new Error(`native archive checksum mismatch: ${file.path}`);
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.copyFileSync(source, destination);
    fs.chmodSync(destination, 0o755);
  }
}

function run(index, count) {
  const manifest = JSON.parse(fs.readFileSync(path.join(bundle, "manifest.json"), "utf8"));
  restore(manifest);
  const inventoryPath = path.join(bundle, "inventory.json");
  if (digest(inventoryPath) !== manifest.inventorySha256)
    throw new Error("native archive inventory checksum mismatch");
  const inventory = JSON.parse(fs.readFileSync(inventoryPath, "utf8"));
  const plan = casePlan(process.env.RUNNER_OS, manifest.entries, inventory, index, count);
  const batches = batchPlan(plan);
  fs.writeFileSync(
    "target/native-rust-shard.json",
    `${JSON.stringify({ index, count, inventory, plan, batches }, null, 2)}\n`,
  );
  fs.writeFileSync("target/native-rust-results.jsonl", "");
  for (const entry of batches) {
    console.log(`${entry.executable} ${entry.args.join(" ")}`);
    const started = Date.now();
    const result = spawnSync(path.resolve(entry.executable), entry.args, {
      stdio: "inherit",
      cwd: path.resolve("crates/no-mistakes"),
    });
    fs.appendFileSync(
      "target/native-rust-results.jsonl",
      `${JSON.stringify({
        ...entry,
        elapsedSeconds: (Date.now() - started) / 1000,
        exitCode: result.status,
        signal: result.signal,
        error: result.error?.message,
      })}\n`,
    );
    completed(result);
  }
}

if (require.main === module) {
  try {
    if (process.argv[2] === "build" && process.argv.length === 3) build();
    else if (process.argv[2] === "run" && process.argv.length === 5)
      run(Number(process.argv[3]), Number(process.argv[4]));
    else throw new Error("usage: native-test-archive.cjs build | run <shard> <count>");
  } catch (error) {
    console.error(error.message);
    process.exitCode = error.status || 1;
  }
}
module.exports = { build, restore, run };
