const { run } = require(process.argv[2]);
try {
  run(Number(process.argv[3]), Number(process.argv[4]), {
    cli: require("node:path").resolve("vitest.cjs"),
    suite: process.argv[5],
  });
} catch (error) {
  console.error(error.message);
  process.exitCode = error.status || 1;
}
