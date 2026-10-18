// No generated fixture files: observe finalization effects through dependency boundaries.
const { join } = require("node:path");
const { readFileSync } = require("node:fs");
const root = process.argv[2],
  scenario = process.argv[3];
const directory = join(__dirname, "pair");
const metadata = JSON.parse(readFileSync(join(directory, "environment.json")));
const runtimePath = join(root, ".github/scripts/hosted-check-runtime.cjs");
const sourcePath = join(root, ".github/scripts/hosted-check-source.cjs");
const runtime = require(runtimePath),
  source = require(sourcePath),
  trace = [];
require.cache[runtimePath].exports = {
  ...runtime,
  hashFile: (path) =>
    path.endsWith(".stdout")
      ? runtime.hashFile(path)
      : scenario === "config"
        ? "mutated"
        : metadata.configSha256,
  command: () => {
    trace.push("tree");
    if (scenario === "tree") throw new Error("dirty tree");
  },
};
require.cache[sourcePath].exports = {
  ...source,
  recordParity: () => true,
  recordCompletion: (_output, value) =>
    trace.push({
      complete: true,
      checkOutcome: value.checkOutcome,
      checkAccepted: value.checkAccepted,
    }),
};
let error;
try {
  require(join(root, ".github/scripts/hosted-check-completion.cjs")).finishMeasurement(
    directory,
    metadata,
    root,
    true,
    false,
  );
} catch (value) {
  error = value.message;
}
console.log(JSON.stringify({ trace, error }));
