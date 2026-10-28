const { mkdirSync, readdirSync, renameSync, statSync } = require("node:fs");
const { join } = require("node:path");

function preserveProfiles(targetDir, mergedProfile) {
  if (!targetDir || !mergedProfile) {
    throw new Error("usage: preserve-rust-profiles.cjs <target-dir> <merged-profdata>");
  }
  if (!statSync(mergedProfile, { throwIfNoEntry: false })?.isFile()) {
    throw new Error(`merged Rust coverage profile is missing: ${mergedProfile}`);
  }
  if (statSync(mergedProfile).size === 0) {
    throw new Error(`merged Rust coverage profile is empty: ${mergedProfile}`);
  }

  const rawProfiles = readdirSync(targetDir, { withFileTypes: true })
    .filter((entry) => entry.isFile() && entry.name.endsWith(".profraw"))
    .map((entry) => entry.name)
    .sort();
  if (rawProfiles.length === 0) return;

  const destination = join(targetDir, "llvm-cov", "raw-profiles");
  mkdirSync(destination, { recursive: true });
  for (const name of rawProfiles) {
    const source = join(targetDir, name);
    const moved = join(destination, name);
    if (statSync(moved, { throwIfNoEntry: false })) {
      throw new Error(`refusing to overwrite preserved Rust profile: ${moved}`);
    }
    renameSync(source, moved);
  }
}

if (require.main === module) {
  try {
    preserveProfiles(process.argv[2], process.argv[3]);
  } catch (error) {
    console.error(`Rust profile preservation failed: ${error.message}`);
    process.exitCode = 1;
  }
}

module.exports = { preserveProfiles };
