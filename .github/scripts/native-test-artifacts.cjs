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

function executionPlan(platform, entries) {
  const { targets } = require("./native-rust-tests.cjs");
  const selected = targets(platform);
  if (entries.length !== selected.length) throw new Error("incomplete native test archive");
  return selected.flatMap(({ selector, cases }, targetIndex) => {
    const entry = entries[targetIndex];
    if (entry.name !== selector[1] || entry.kind !== selector[0].slice(2))
      throw new Error(`native archive target mismatch: ${selector.join(" ")}`);
    return cases.map((args) => ({
      executable: entry.executable,
      args: args.filter((arg) => arg !== "--"),
    }));
  });
}

function parseListing(stdout) {
  const names = [];
  let summary;
  for (const line of stdout.split(/\r?\n/).filter((line) => line.trim())) {
    const test = line.match(/^(.+): test$/);
    if (test) names.push(test[1]);
    else if (/^\d+ tests?, 0 benchmarks?$/.test(line) && summary === undefined)
      summary = Number(line.split(" ")[0]);
    else throw new Error(`malformed native test listing: ${line}`);
  }
  if (!names.length) throw new Error("native test filter selected no tests");
  if (new Set(names).size !== names.length || (summary !== undefined && summary !== names.length))
    throw new Error("native test listing contains duplicate names or an incorrect count");
  return names.sort();
}

function casePlan(platform, entries, inventory, index, count) {
  shard(index, count);
  const groups = executionPlan(platform, entries);
  if (!Array.isArray(inventory) || inventory.length !== groups.length)
    throw new Error("incomplete native test inventory");
  const cases = [];
  const seen = new Set();
  groups.forEach((group, groupIndex) => {
    const listed = inventory[groupIndex];
    if (
      listed.executable !== group.executable ||
      JSON.stringify(listed.args) !== JSON.stringify(group.args)
    )
      throw new Error("native test inventory differs from the original filters");
    if (!Array.isArray(listed.tests) || !listed.tests.length)
      throw new Error("native test filter selected no tests");
    for (const name of listed.tests) {
      if (typeof name !== "string" || !name || /[\r\n]/.test(name))
        throw new Error("malformed native test inventory name");
      const identity = `${group.executable}\0${name}`;
      if (seen.has(identity)) throw new Error(`native test selected more than once: ${name}`);
      seen.add(identity);
      const filters = group.args.filter((arg) => !arg.startsWith("--"));
      if (filters.some((filter) => !name.includes(filter)))
        throw new Error(`native test is outside its original filter: ${name}`);
      cases.push({
        executable: group.executable,
        name,
        args: [
          name,
          "--exact",
          ...group.args.filter((arg) => arg.startsWith("--") && arg !== "--exact"),
        ],
      });
    }
  });
  cases.sort((a, b) => {
    const left = `${a.executable}\0${a.name}`;
    const right = `${b.executable}\0${b.name}`;
    return left < right ? -1 : left > right ? 1 : 0;
  });
  return cases.filter((_, offset) => offset % count === index - 1);
}

module.exports = {
  collectArtifacts,
  relativeExecutable,
  digest,
  executionPlan,
  parseListing,
  casePlan,
};
