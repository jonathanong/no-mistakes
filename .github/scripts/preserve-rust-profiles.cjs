const { lstatSync, mkdirSync, readdirSync, renameSync, statSync } = require("node:fs");
const { basename, isAbsolute, join, relative, resolve, sep } = require("node:path");

function preserveProfiles(targetDir, workspaceRoot, archiveDir) {
  if (!targetDir || !workspaceRoot || !archiveDir) {
    throw new Error(
      "usage: preserve-rust-profiles.cjs <target-dir> <workspace-root> <archive-dir>",
    );
  }
  targetDir = resolve(targetDir);
  archiveDir = resolve(archiveDir);
  const insideTarget = relative(targetDir, archiveDir);
  const outsideTarget =
    insideTarget === ".." || insideTarget.startsWith(`..${sep}`) || isAbsolute(insideTarget);
  if (!outsideTarget) {
    throw new Error(`archive directory must be outside the Cargo target directory: ${archiveDir}`);
  }

  const mergedProfile = join(targetDir, `${basename(resolve(workspaceRoot))}.profdata`);
  if (!statSync(mergedProfile, { throwIfNoEntry: false })?.isFile()) {
    throw new Error(`merged Rust coverage profile is missing: ${mergedProfile}`);
  }
  if (statSync(mergedProfile).size === 0) {
    throw new Error(`merged Rust coverage profile is empty: ${mergedProfile}`);
  }

  const entries = readdirSync(targetDir, { withFileTypes: true }).filter((entry) =>
    entry.name.endsWith(".profraw"),
  );
  for (const entry of entries) {
    if (!entry.isFile()) {
      throw new Error(`refusing nonregular Rust profile entry: ${join(targetDir, entry.name)}`);
    }
  }
  const rawProfiles = entries.map(({ name }) => name).sort();
  if (rawProfiles.length === 0) return;

  for (const name of rawProfiles) {
    const destination = join(archiveDir, name);
    if (lstatSync(destination, { throwIfNoEntry: false })) {
      throw new Error(`refusing to overwrite preserved Rust profile: ${destination}`);
    }
  }

  mkdirSync(archiveDir, { recursive: true });
  for (const name of rawProfiles) {
    renameSync(join(targetDir, name), join(archiveDir, name));
  }
  if (readdirSync(targetDir).some((name) => name.endsWith(".profraw"))) {
    throw new Error(`root Rust raw profiles remain after preservation in ${targetDir}`);
  }
}

if (require.main === module) {
  try {
    preserveProfiles(process.argv[2], process.argv[3], process.argv[4]);
  } catch (error) {
    console.error(`Rust profile preservation failed: ${error.message}`);
    process.exitCode = 1;
  }
}

module.exports = { preserveProfiles };
