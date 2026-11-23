"use strict";

const { unwrapExpression } = require("./async-ast");
const { literalString, propertyName } = require("./logical-call");

const FS_PATH_METHODS = new Set([
  "access",
  "appendFile",
  "chmod",
  "chown",
  "glob",
  "lchmod",
  "lchown",
  "lstat",
  "lutimes",
  "mkdir",
  "open",
  "opendir",
  "readFile",
  "readdir",
  "readlink",
  "realpath",
  "rm",
  "rmdir",
  "stat",
  "statfs",
  "truncate",
  "unlink",
  "utimes",
  "writeFile",
]);
const FS_ONLY_PATH_METHODS = new Set([
  "createReadStream",
  "createWriteStream",
  "exists",
  "existsSync",
  "openAsBlob",
  "unwatchFile",
  "watchFile",
]);
const FS_TWO_PATH_METHODS = new Set(["copyFile", "cp", "link", "rename", "symlink"]);
const FS_NAMESPACES = new Set(["fs", "fs/promises", "fs.promises"]);
const PROCESS_ARGV_METHODS = new Set(["spawn", "spawnSync", "execFile", "execFileSync"]);
const PROCESS_SHELL_METHODS = new Set(["exec", "execSync"]);

function filesystemPathCount(namespace, name) {
  if (name === "native" && (namespace === "fs.realpath" || namespace === "fs.realpathSync")) {
    return 1;
  }
  if (!FS_NAMESPACES.has(namespace)) return 0;
  if (name === "watch" || (namespace === "fs" && FS_ONLY_PATH_METHODS.has(name))) return 1;
  const method = namespace === "fs" ? name.replace(/Sync$/, "") : name;
  if (FS_TWO_PATH_METHODS.has(method)) return 2;
  return FS_PATH_METHODS.has(method) ? 1 : 0;
}

function cwdOption(options) {
  if (options?.type !== "ObjectExpression") return null;
  for (const property of options.properties.toReversed()) {
    if (property.type !== "Property") return null;
    const name = property.computed
      ? literalString(unwrapExpression(property.key))
      : propertyName(property.key);
    if (name === null) return null;
    if (name === "cwd") {
      return property.kind === "init" && !property.method ? property.value : null;
    }
  }
  return null;
}

function readProcessContainer(input, origin) {
  const direct = unwrapExpression(input);
  const value = origin(direct);
  return { value, readPath: direct === value ? origin : unwrapExpression };
}

function forEachProcessPath(args, method, origin, visit) {
  const [command, argsOrOptions, options] = args;
  if (command?.type === "SpreadElement") return;
  visit(origin(command));
  if (argsOrOptions?.type === "SpreadElement") return;

  const second = readProcessContainer(argsOrOptions, origin);
  if (PROCESS_SHELL_METHODS.has(method) || second.value?.type === "ObjectExpression") {
    visit(second.readPath(cwdOption(second.value)));
    return;
  }
  if (second.value?.type === "ArrayExpression") {
    for (const argument of second.value.elements) visit(second.readPath(argument));
  }
  const third = readProcessContainer(options, origin);
  visit(third.readPath(cwdOption(third.value)));
}

function forEachPathArgument(args, binding, origin, visit) {
  if (!binding) return;
  const separator = binding.lastIndexOf(".");
  const namespace = binding.slice(0, separator);
  const method = binding.slice(separator + 1);
  const pathCount = filesystemPathCount(namespace, method);
  if (pathCount > 0) {
    for (const argument of args.slice(0, pathCount)) {
      if (argument.type === "SpreadElement") break;
      visit(origin(argument));
    }
  } else if (
    namespace === "child_process" &&
    (PROCESS_ARGV_METHODS.has(method) || PROCESS_SHELL_METHODS.has(method))
  ) {
    forEachProcessPath(args, method, origin, visit);
  }
}

module.exports = { forEachPathArgument };
