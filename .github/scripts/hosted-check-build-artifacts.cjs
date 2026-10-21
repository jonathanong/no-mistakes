const { createHash } = require("node:crypto");
const {
  copyFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} = require("node:fs");
const { isAbsolute, join, relative, resolve, sep } = require("node:path");

const TARGET = "x86_64-unknown-linux-gnu";
const STUB = "// NO_MISTAKES_HOSTED_DEPENDENCY_STUB\n";
function hashFile(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function externalTarget(source, target) {
  const path = resolve(target || "");
  const inside = relative(resolve(source), path);
  if (
    !target ||
    inside === "" ||
    (inside !== ".." && !inside.startsWith(`..${sep}`) && !isAbsolute(inside))
  ) {
    throw new Error("CARGO_TARGET_DIR must be outside the source checkout");
  }
  return path;
}

function ownedArtifact(area, name) {
  if ([".fingerprint", "build"].includes(area)) return /^no-mistakes-/.test(name);
  return /^(?:lib)?no_mistakes(?:-|\.)/.test(name) || /^no-mistakes(?:\.|$)/.test(name);
}

function removeStubArtifacts(target) {
  let removed = 0;
  for (const profile of [join(target, "release"), join(target, TARGET, "release")]) {
    for (const area of ["", ".fingerprint", "build", "deps"]) {
      const directory = join(profile, area);
      if (!existsSync(directory)) continue;
      for (const name of readdirSync(directory)) {
        if (ownedArtifact(area, name)) {
          rmSync(join(directory, name), { recursive: true, force: true });
          removed++;
        }
      }
    }
  }
  return removed;
}

function stageArtifacts(source, target, output, provenance, command) {
  mkdirSync(output, { recursive: true });
  const profile = join(target, TARGET, "release");
  copyFileSync(join(profile, "no-mistakes"), join(output, "no-mistakes"));
  const dependencies = join(profile, "deps");
  const libraries = readdirSync(dependencies).filter((name) => /^libnum_cpus-.*\.rlib$/.test(name));
  if (libraries.length !== 1)
    throw new Error("Expected one exact num_cpus dependency for the CPU probe");
  const probeSource = join(output, "cpu-probe.rs");
  writeFileSync(probeSource, 'fn main() { println!("{}", num_cpus::get()); }\n');
  command(
    "rustc",
    [
      probeSource,
      "--edition=2021",
      "--target",
      TARGET,
      "-L",
      `dependency=${dependencies}`,
      "--extern",
      `num_cpus=${join(dependencies, libraries[0])}`,
      "-o",
      join(output, "cpu-probe"),
    ],
    source,
  );
  const binarySha256 = hashFile(join(output, "no-mistakes"));
  const cpuProbeSha256 = hashFile(join(output, "cpu-probe"));
  writeFileSync(
    join(output, "SHA256SUMS"),
    `${binarySha256}  no-mistakes\n${cpuProbeSha256}  cpu-probe\n`,
  );
  const version = command(join(output, "no-mistakes"), ["--version"], source);
  writeFileSync(
    join(output, "provenance.json"),
    `${JSON.stringify({ ...provenance, version, binarySha256, cpuProbeSha256 }, null, 2)}\n`,
  );
}

module.exports = {
  TARGET,
  STUB,
  hashFile,
  externalTarget,
  ownedArtifact,
  removeStubArtifacts,
  stageArtifacts,
};
