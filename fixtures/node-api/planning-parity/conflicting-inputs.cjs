const assert = require("node:assert/strict");
const { impactedChecks } = require("../../../packages/no-mistakes/index.js");

async function run() {
  const root = process.argv[2];
  const sources = [
    { diff: "inline text" },
    { diffFile: "missing.patch" },
    { diffStdin: true },
    { diffCommand: "no-mistakes-example-missing-command" },
  ];
  for (const [index, first] of sources.entries()) {
    for (const second of sources.slice(index + 1)) {
      await assert.rejects(
        impactedChecks({ root, ...first, ...second }),
        /diff, diffFile, diffStdin, and diffCommand are mutually exclusive/,
      );
    }
  }
  process.stdout.write("6 conflicts rejected\n");
}
run().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
