const { run } = require(process.argv[2]);
try {
  run(Number(process.argv[3]), Number(process.argv[4]), undefined, require("node:path").resolve("vitest.cjs"));
} catch (error) {
  console.error(error.message);
  process.exitCode = error.status || 1;
}
