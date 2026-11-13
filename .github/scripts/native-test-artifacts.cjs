const fs = require("node:fs");
const path = require("node:path");
const { createHash } = require("node:crypto");

function collectArtifacts(stream) {
  const artifacts = [];
  require("node:readline")
    .createInterface({ input: stream })
    .on("line", (line) => {
      if (!line.startsWith("{")) return;
      const message = JSON.parse(line);
      if (message.reason === "compiler-artifact" && message.executable)
        artifacts.push({
          executable: message.executable,
          name: message.target.name,
          kind: message.target.kind,
          test: message.profile.test,
        });
    });
  return artifacts;
}

function relativeExecutable(workspace, executable) {
  const relative = path.relative(workspace, executable);
  if (path.isAbsolute(relative) || relative.startsWith(`..${path.sep}`) || relative === "..")
    throw new Error(`native executable is outside workspace: ${executable}`);
  if (!relative.startsWith(`target${path.sep}debug${path.sep}`))
    throw new Error(`expected a debug native executable: ${relative}`);
  return relative.split(path.sep).join("/");
}

function digest(file) {
  return createHash("sha256").update(fs.readFileSync(file)).digest("hex");
}

function shard(value, count) {
  if (
    !Number.isInteger(value) ||
    !Number.isInteger(count) ||
    count < 1 ||
    value < 1 ||
    value > count
  )
    throw new Error("native shard must satisfy 1 <= shard <= count");
}

function executionPlan(platform, entries, index = 1, count = 1) {
  shard(index, count);
  const { targets } = require("./native-rust-tests.cjs");
  const selected = targets(platform);
  if (entries.length !== selected.length) throw new Error("incomplete native test archive");
  return selected
    .flatMap(({ selector, cases }, targetIndex) => {
      const entry = entries[targetIndex];
      if (entry.name !== selector[1] || entry.kind !== selector[0].slice(2))
        throw new Error(`native archive target mismatch: ${selector.join(" ")}`);
      return cases.map((args) => ({
        executable: entry.executable,
        args: args.filter((arg) => arg !== "--"),
      }));
    })
    .filter((_, caseIndex) => caseIndex % count === index - 1);
}

module.exports = { collectArtifacts, relativeExecutable, digest, executionPlan };
