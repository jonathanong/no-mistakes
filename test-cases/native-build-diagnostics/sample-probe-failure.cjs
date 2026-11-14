const fs = require("node:fs");
const path = require("node:path");

const mode = process.argv[2];
if (mode !== "timeout") {
  console.error("usage: sample-probe-failure.cjs timeout");
  process.exit(1);
}

fs.mkdirSync(path.resolve("target/native-build-diagnostics"), { recursive: true });
const { probe } = require("../../.github/scripts/native-build-probes.cjs");
const { sample } = require("../../.github/scripts/build-native.cjs");
const wait = path.join(__dirname, "diagnostic-wait.cjs");

sample((command, args, timeout) => {
  if (command !== "ps" && command !== "powershell") return probe(command, args, timeout);
  return probe(process.execPath, [wait], 100);
});
