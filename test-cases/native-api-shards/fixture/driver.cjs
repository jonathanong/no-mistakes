const path = require("node:path");
const { run } = require(process.argv[2]);
try {
  run(
    Number(process.argv[3]),
    Number(process.argv[4]),
    undefined,
    path.resolve("vitest.cjs"),
    process.argv[5] === "linux"
      ? require(path.join(path.dirname(process.argv[2]), "native-api-linux-files.cjs"))
      : undefined,
  );
} catch (error) {
  console.error(error.message);
  process.exitCode = error.status || 1;
}
