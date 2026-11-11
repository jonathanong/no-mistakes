const fs = require("node:fs");
const path = require("node:path");

function captureBuildOutput(child, directory) {
  const log = fs.createWriteStream(path.join(directory, "cargo.log"));
  let logFailed = false;
  log.on("error", (error) => {
    logFailed = true;
    console.warn(`Native build cargo.log: ${error.message}`);
    try {
      fs.writeFileSync(
        path.join(directory, "log-error.txt"),
        `${error.code || "ERROR"}: ${error.message}\n`,
      );
    } catch (diagnosticError) {
      console.warn(`Native build diagnostics: ${diagnosticError.message}`);
    }
  });
  for (const [source, destination] of [
    [child.stdout, process.stdout],
    [child.stderr, process.stderr],
  ]) {
    source.on("data", (data) => {
      destination.write(data);
      if (!logFailed && !log.destroyed) log.write(data);
    });
  }
  return () => {
    if (!logFailed && !log.destroyed) log.end();
  };
}

module.exports = { captureBuildOutput };
