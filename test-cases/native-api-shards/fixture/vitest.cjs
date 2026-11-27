const fs = require("node:fs");
const path = require("node:path");
const args = process.argv.slice(2);
const inventoryPath =
  process.env.NATIVE_API_FIXTURE_SUITE === "linux" ? "linux-inventory.json" : "inventory.json";
fs.appendFileSync("commands.jsonl", `${JSON.stringify(args)}\n`);
if (process.env.NATIVE_API_FIXTURE_EXIT) process.exit(Number(process.env.NATIVE_API_FIXTURE_EXIT));
if (args[0] === "list") {
  if (!args.includes("--no-static-parse"))
    throw new Error("dynamic test imports require collection");
  fs.copyFileSync(inventoryPath, "target/native-api-inventory.json");
} else {
  if (args.some((arg) => arg.startsWith("--testNamePattern=")))
    throw new Error("test names must be read from the shard plan");
  const config = args[args.indexOf("--config") + 1];
  if (path.basename(config) !== "vitest-native-api.config.cjs")
    throw new Error("native API tests require the shard plan configuration");
  const { readShardTestPattern } = require(
    path.join(path.dirname(config), "native-api-shards.cjs"),
  );
  const pattern = readShardTestPattern();
  const selected = JSON.parse(fs.readFileSync(inventoryPath, "utf8")).filter((row) =>
    pattern.test(row.name.replaceAll(" > ", " ")),
  );
  fs.writeFileSync("target/native-api-tests.json", JSON.stringify(selected));
}
