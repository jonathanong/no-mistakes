// Check executed insta references after the full workspace test run. A static
// name scan would miss dynamically named snapshots and references in helpers.
const { execFileSync } = require("node:child_process");
const { readFileSync, realpathSync, statSync, mkdtempSync, rmSync } = require("node:fs");
const { tmpdir } = require("node:os");
const path = require("node:path");

function canonical(filename) {
  let resolved = path.resolve(filename);
  try {
    resolved = realpathSync.native(resolved);
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
  return process.platform === "win32" ? resolved.toLowerCase() : resolved;
}

function unreferencedSnapshots(files, references, packageRoots) {
  const used = new Set(references.map(canonical));
  const roots = packageRoots.map(canonical);
  // Git's cached inventory retains deleted paths until their removal is staged.
  const present = files.filter((file) => statSync(file, { throwIfNoEntry: false })?.isFile());
  const unique = new Map(present.map((file) => [canonical(file), path.resolve(file)]));
  return [...unique]
    .filter(([file]) =>
      roots.some((root) => {
        const relative = path.relative(root, file);
        return (
          relative !== ".." && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative)
        );
      }),
    )
    .filter(([file]) => !used.has(file))
    .map(([, file]) => file)
    .sort();
}

function snapshotFiles(workspace) {
  const args = ["ls-files", "-z", "--cached", "--others", "--exclude-standard", "--", "*.snap"];
  const options = {
    cwd: workspace,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    env: { ...process.env, LC_ALL: "C" },
  };
  let listing;
  try {
    listing = execFileSync("git", args, options);
  } catch (error) {
    if (!String(error.stderr).includes("not a git repository")) throw error;
    // Release validation uses a git archive. An empty, temporary Git index
    // applies the archive's ignore files without adding .git to its source tree
    // or traversing ignored build outputs with a separate filesystem walker.
    const inventory = mkdtempSync(path.join(tmpdir(), "no-mistakes-snapshot-inventory-"));
    try {
      execFileSync("git", ["init", "--bare", "--quiet", "--template=", inventory], options);
      listing = execFileSync(
        "git",
        ["--git-dir", inventory, "--work-tree", workspace, "-c", "core.bare=false", ...args],
        options,
      );
    } finally {
      rmSync(inventory, { recursive: true, force: true });
    }
  }
  return listing
    .split("\0")
    .filter(Boolean)
    .map((file) => path.resolve(workspace, file));
}

function check(referenceFile) {
  const metadata = JSON.parse(
    execFileSync("cargo", ["metadata", "--locked", "--no-deps", "--format-version", "1"], {
      encoding: "utf8",
    }),
  );
  const members = new Set(metadata.workspace_members);
  const roots = metadata.packages
    .filter((entry) => members.has(entry.id))
    .map((entry) => path.dirname(entry.manifest_path));
  if (!roots.length) throw new Error("Cargo workspace has no packages to audit");
  const workspace = metadata.workspace_root;
  // Git supplies the inventory, including new snapshots, without walking build
  // outputs. Snapshot fixtures outside workspace packages are not test artifacts.
  const files = snapshotFiles(workspace);
  const references = readFileSync(referenceFile, "utf8").split(/\r?\n/).filter(Boolean);
  const orphaned = unreferencedSnapshots(files, references, roots);
  if (orphaned.length) {
    throw new Error(
      `Unreferenced Rust snapshots; restore their tests or remove the files:\n${orphaned
        .map((file) => path.relative(workspace, file).split(path.sep).join("/"))
        .join("\n")}`,
    );
  }
}

if (require.main === module) {
  try {
    if (process.argv.length !== 3)
      throw new Error("Usage: rust-snapshot-references.cjs <references>");
    check(process.argv[2]);
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}

module.exports = { unreferencedSnapshots, snapshotFiles };
