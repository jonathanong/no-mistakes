const { execFileSync } = require("node:child_process");
const { createHash, randomUUID } = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");

const manifestName = "cargo-cache-inputs.json";
const past = new Date("2000-01-01T00:00:00.000Z");
const buffer = Buffer.alloc(64 * 1024);
function git(root, args) {
  const env = { ...process.env, GIT_OPTIONAL_LOCKS: "0" };
  for (const name of ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_COMMON_DIR"])
    delete env[name];
  return execFileSync("git", ["-C", root, ...args], {
    encoding: "utf8",
    env,
    maxBuffer: 64 * 1024 * 1024,
  });
}
function workspaceRoot(workspace) {
  if (!workspace || !fs.lstatSync(workspace).isDirectory())
    throw new Error("Workspace must be a regular Git directory, not a symlink");
  const root = fs.realpathSync(workspace);
  const top = fs.realpathSync(git(root, ["rev-parse", "--show-toplevel"]).trim());
  if (top !== root) throw new Error("Workspace must be the Git top-level directory");
  return root;
}
function safePath(value) {
  if (
    typeof value !== "string" ||
    !value ||
    value.includes("\\") ||
    value.includes("\0") ||
    path.posix.isAbsolute(value) ||
    path.win32.isAbsolute(value) ||
    value.split("/").some((part) => !part || part === "." || part === ".." || part === ".git")
  )
    throw new Error(`Unsafe input path: ${JSON.stringify(value)}`);
  return value;
}
function inventory(root) {
  const files = [];
  const seen = new Set();
  for (const record of git(root, ["ls-files", "--stage", "-z"]).split("\0").filter(Boolean)) {
    const separator = record.indexOf("\t");
    const header = record.slice(0, separator).match(/^(\d{6}) [0-9a-f]+ ([0-3])$/);
    if (!header) throw new Error("Malformed Git input inventory");
    if (header[2] !== "0") throw new Error("Cannot record inputs with unresolved Git conflicts");
    if (!["100644", "100755"].includes(header[1])) continue;
    const name = safePath(record.slice(separator + 1));
    if (seen.has(name)) throw new Error(`Duplicate Git input: ${name}`);
    seen.add(name);
    files.push(name);
  }
  return files.sort();
}
function inspector(root) {
  const directories = new Set([root]);
  return (name) => {
    const parts = safePath(name).split("/");
    let parent = root;
    for (const part of parts.slice(0, -1)) {
      parent = path.join(parent, part);
      if (directories.has(parent)) continue;
      const stat = fs.lstatSync(parent, { throwIfNoEntry: false });
      if (!stat) return undefined;
      if (!stat.isDirectory())
        throw new Error(`Refusing symlink or non-directory input: ${parent}`);
      directories.add(parent);
    }
    const file = path.join(root, ...parts);
    const stat = fs.lstatSync(file, { throwIfNoEntry: false });
    if (!stat) return undefined;
    if (!stat.isFile()) throw new Error(`Refusing symlink or nonregular input: ${file}`);
    return { file, stat };
  };
}
function digest(file) {
  const descriptor = fs.openSync(file, fs.constants.O_RDONLY | (fs.constants.O_NOFOLLOW || 0));
  const hash = createHash("sha256");
  try {
    let count;
    while ((count = fs.readSync(descriptor, buffer)) > 0) hash.update(buffer.subarray(0, count));
    return hash.digest("hex");
  } finally {
    fs.closeSync(descriptor);
  }
}
function manifestPath(directory) {
  const stat = fs.lstatSync(directory, { throwIfNoEntry: false });
  if (stat && !stat.isDirectory()) throw new Error("Sidecar directory must not be a symlink");
  const file = path.join(directory, manifestName);
  const entry = fs.lstatSync(file, { throwIfNoEntry: false });
  if (entry && !entry.isFile()) throw new Error("Sidecar manifest must be a regular file");
  return file;
}
function capture(workspace, directory) {
  const root = workspaceRoot(workspace);
  const destination = manifestPath(directory);
  const inspect = inspector(root);
  const files = inventory(root).flatMap((name) => {
    const item = inspect(name);
    return item ? [{ path: name, sha256: digest(item.file) }] : [];
  });
  fs.mkdirSync(directory, { recursive: true });
  const temporary = `${destination}.${randomUUID()}.tmp`;
  try {
    fs.writeFileSync(temporary, `${JSON.stringify({ version: 1, workspace: root, files })}\n`, {
      flag: "wx",
    });
    fs.renameSync(temporary, destination);
  } finally {
    fs.rmSync(temporary, { force: true });
  }
  return { captured: files.length };
}
function restore(workspace, directory) {
  const root = workspaceRoot(workspace);
  const manifest = manifestPath(directory);
  if (!fs.existsSync(manifest)) return { cached: false, restored: 0, fresh: 0 };
  const data = JSON.parse(fs.readFileSync(manifest, "utf8"));
  if (data.version !== 1 || !Array.isArray(data.files))
    throw new Error("Unsupported input manifest");
  const hashes = new Map();
  for (const row of data.files) {
    const name = safePath(row.path);
    if (!/^[0-9a-f]{64}$/.test(row.sha256 || "")) throw new Error(`Invalid input hash: ${name}`);
    if (hashes.has(name)) throw new Error(`Duplicate input manifest path: ${name}`);
    hashes.set(name, row.sha256);
  }
  if (data.workspace !== root)
    throw new Error("Input manifest workspace differs from this checkout");
  const inspect = inspector(root);
  for (const name of hashes.keys()) inspect(name);
  const updates = inventory(root).flatMap((name) => {
    const item = inspect(name);
    if (!item) return [];
    return [{ ...item, matched: hashes.has(name) && digest(item.file) === hashes.get(name) }];
  });
  const now = new Date();
  for (const item of updates) fs.utimesSync(item.file, item.stat.atime, item.matched ? past : now);
  const restored = updates.filter((item) => item.matched).length;
  return { cached: true, restored, fresh: updates.length - restored };
}
if (require.main === module) {
  try {
    const [mode, workspace, directory] = process.argv.slice(2);
    if (process.argv.length !== 5 || !["capture", "restore"].includes(mode))
      throw new Error(
        "Usage: cargo-cache-inputs.cjs {capture|restore} <workspace> <sidecar-directory>",
      );
    console.log(JSON.stringify({ capture, restore }[mode](workspace, directory)));
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
module.exports = { capture, restore, past, manifestName };
