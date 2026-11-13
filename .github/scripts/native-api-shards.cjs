const fs = require("node:fs");
const path = require("node:path");
const { spawnSync } = require("node:child_process");

const files = [
  "real-napi-api",
  "unused-disable-directives-api",
  "sql-verdicts-api",
  "postgres-procedural-occurrence-api",
  "postgres-procedural-loop-coverage-api",
  "guarded-array-receiver-api",
  "forbidden-dependencies-globs-api",
].map((name) => `packages/no-mistakes/scripts/${name}.test.js`);

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
  const names = [
    ...new Set(inventory.filter((row) => selected.has(key(row.name))).map((row) => row.name)),
  ];
  if (!names.length) throw new Error("native API shard has no collected tests");
  const pattern = `^(?:${names.map((name) => name.split(" > ").map(escape).join("(?: > | )")).join("|")})$`;
  if (pattern.length > 24000)
    throw new Error(
      "native API shard pattern exceeds Windows command budget; increase shard count",
    );
  return { pattern, names, total: inventory.length };
}

function run(
  index,
  count,
  execute = spawnSync,
  cli = path.join(path.dirname(require.resolve("vitest/package.json")), "vitest.mjs"),
) {
  fs.mkdirSync("target", { recursive: true });
  const invoke = (args) => {
    const result = execute(process.execPath, [cli, ...args], {
      stdio: "inherit",
      env: process.env,
    });
    if (result.error) throw result.error;
    if (result.status !== 0) {
      const error = new Error(`native API Vitest failed: ${result.signal || result.status}`);
      error.status = result.status ?? 1;
      throw error;
    }
  };
  invoke([
    "list",
    ...files,
    "--no-static-parse",
    "--json=target/native-api-inventory.json",
    "--coverage=false",
  ]);
  const inventory = JSON.parse(fs.readFileSync("target/native-api-inventory.json", "utf8"));
  const plan = partition(inventory, index, count);
  fs.writeFileSync(
    "target/native-api-shard.json",
    `${JSON.stringify({ index, count, ...plan }, null, 2)}\n`,
  );
  invoke([
    "run",
    ...files,
    `--testNamePattern=${plan.pattern}`,
    "--reporter=default",
    "--reporter=json",
    "--outputFile.json=target/native-api-tests.json",
    "--coverage=false",
  ]);
}

if (require.main === module) {
  try {
    if (process.argv.length !== 4) throw new Error("usage: native-api-shards.cjs <shard> <count>");
    run(Number(process.argv[2]), Number(process.argv[3]));
  } catch (error) {
    console.error(error.message);
    process.exitCode = error.status || 1;
  }
}
module.exports = { files, key, partition, run };
