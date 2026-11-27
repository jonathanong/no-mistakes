const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const { platformFiles, linuxFiles } = require("./native-api-files.cjs");

function key(name) {
  return name.replaceAll(" > ", " ");
}

function escape(name) {
  return name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function partition(inventory, index, count) {
  if (!Number.isInteger(index) || !Number.isInteger(count) || index < 1 || count < index)
    throw new Error("native API shard must satisfy 1 <= shard <= count");
  if (
    !Array.isArray(inventory) ||
    inventory.length === 0 ||
    inventory.some(
      (row) => typeof row.name !== "string" || !row.name || typeof row.file !== "string",
    )
  )
    throw new Error("native API inventory must contain collected test names and files");
  const keys = [...new Set(inventory.map((row) => key(row.name)))].sort();
  const selected = new Set(keys.filter((_, offset) => offset % count === index - 1));
  const selectedRows = inventory.filter((row) => selected.has(key(row.name)));
  const names = [...new Set(selectedRows.map((row) => row.name))];
  const files = [...new Set(selectedRows.map((row) => row.file))];
  if (!names.length) throw new Error("native API shard has no collected tests");
  const pattern = `^(?:${names.map((name) => name.split(" > ").map(escape).join("(?: > | )")).join("|")})$`;
  return { pattern, names, files, total: inventory.length };
}

function selectRunFiles(files, selectedFiles) {
  const known = new Map(files.map((file) => [path.resolve(file), file]));
  const matched = new Set();
  for (const file of selectedFiles) {
    const resolved = known.get(path.resolve(file));
    if (!resolved) return files;
    matched.add(resolved);
  }
  return matched.size ? files.filter((file) => matched.has(file)) : files;
}

function readShardTestPattern() {
  const { pattern } = JSON.parse(fs.readFileSync("target/native-api-shard.json", "utf8"));
  if (typeof pattern !== "string" || pattern.length === 0)
    throw new Error("native API shard plan requires a test filter");
  return new RegExp(pattern);
}

function run(
  index,
  count,
  {
    execute = spawnSync,
    cli = path.join(path.dirname(require.resolve("vitest/package.json")), "vitest.mjs"),
    suite = "platform",
  } = {},
) {
  if (suite !== "platform" && suite !== "linux")
    throw new Error("native API suite must be platform or linux");
  const files = suite === "linux" ? linuxFiles : platformFiles;
  fs.mkdirSync("target", { recursive: true });
  const invoke = (args, extraEnv = {}) => {
    const result = execute(process.execPath, [cli, ...args], {
      stdio: "inherit",
      env: { ...process.env, ...extraEnv },
    });
    if (result.error) throw result.error;
    if (result.status !== 0) {
      const error = new Error(`native API Vitest failed: ${result.signal || result.status}`);
      error.status = result.status ?? 1;
      throw error;
    }
  };
  invoke(
    [
      "list",
      ...files,
      "--no-static-parse",
      "--json=target/native-api-inventory.json",
      "--coverage=false",
    ],
    { NO_MISTAKES_VITEST_LIST: "1" },
  );
  const inventory = JSON.parse(fs.readFileSync("target/native-api-inventory.json", "utf8"));
  const plan = partition(inventory, index, count);
  const runFiles = selectRunFiles(files, plan.files);
  fs.writeFileSync(
    "target/native-api-shard.json",
    `${JSON.stringify({ index, count, ...plan }, null, 2)}\n`,
  );
  invoke([
    "run",
    ...runFiles,
    "--config",
    path.join(__dirname, "vitest-native-api.config.cjs"),
    "--reporter=default",
    "--reporter=json",
    "--outputFile.json=target/native-api-tests.json",
    "--coverage=false",
  ]);
}

if (require.main === module) {
  try {
    if (process.argv.length < 4 || process.argv.length > 5)
      throw new Error("usage: native-api-shards.cjs <shard> <count> [linux]");
    if (process.argv[4] && process.argv[4] !== "linux")
      throw new Error("native API CLI suite must be linux when supplied");
    run(Number(process.argv[2]), Number(process.argv[3]), { suite: process.argv[4] });
  } catch (error) {
    console.error(error.message);
    process.exitCode = error.status || 1;
  }
}
module.exports = { key, partition, readShardTestPattern, run };
