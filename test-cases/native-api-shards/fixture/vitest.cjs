const fs = require("node:fs");
const args = process.argv.slice(2);
fs.appendFileSync("commands.jsonl", `${JSON.stringify(args)}\n`);
if (process.env.NATIVE_API_FIXTURE_EXIT) process.exit(Number(process.env.NATIVE_API_FIXTURE_EXIT));
if (args[0] === "list") {
  if (!args.includes("--no-static-parse")) throw new Error("dynamic test imports require collection");
  fs.copyFileSync("inventory.json", "target/native-api-inventory.json");
} else {
  const pattern = new RegExp(args.find((arg) => arg.startsWith("--testNamePattern=")).slice("--testNamePattern=".length));
  const selected = require("./inventory.json").filter((row) => pattern.test(row.name.replaceAll(" > ", " ")));
  fs.writeFileSync("target/native-api-tests.json", JSON.stringify(selected));
}
