const fs = require("node:fs");
const path = require("node:path");

function captureBuildOutput(child, directory, options = {}) {
  const targets = [];
  let log;
  let logFailed = false;
  let cancelled = false;
  let abandonPending = false;
  let finished = false;
  function detach(target) {
    if (!target.attached) return;
    target.attached = false;
    for (const source of target.sources) {
      source.unpipe(target.destination);
      if (!targets.some((other) => other.attached && other.sources.includes(source)))
        source.resume();
    }
  }
  function logError(error) {
    if (logFailed) return;
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
  }
  function attach(destination, sources, isLog = false) {
    const target = { destination, sources, attached: true };
    targets.push(target);
    destination.on("error", (error) => {
      detach(target);
      if (isLog) logError(error);
    });
    destination.on("close", () => {
      detach(target);
      if (isLog && !destination.writableFinished && !cancelled)
        logError(new Error("cargo.log closed before finishing"));
    });
    for (const source of sources) source.pipe(destination, { end: false });
  }
  attach(options.stdout || process.stdout, [child.stdout]);
  attach(options.stderr || process.stderr, [child.stderr]);
  try {
    log = (options.createLog || fs.createWriteStream)(path.join(directory, "cargo.log"));
    attach(log, [child.stdout, child.stderr], true);
  } catch (error) {
    logError(error);
  }
  function finish() {
    if (finished) return;
    finished = true;
    if (log && !logFailed && !log.destroyed) log.end();
  }
  function releasePending() {
    if (!abandonPending) return;
    for (const target of targets) {
      const destination = target.destination;
      if (target.attached && (destination.writableNeedDrain || destination.writableLength > 0)) {
        detach(target);
        destination.destroy();
      }
    }
  }
  finish.cancel = () => {
    if (cancelled) return;
    cancelled = true;
    for (const source of [child.stdout, child.stderr]) source.on("data", releasePending);
    setTimeout(() => {
      abandonPending = true;
      releasePending();
    }, 1000).unref();
  };
  return finish;
}

module.exports = { captureBuildOutput };
