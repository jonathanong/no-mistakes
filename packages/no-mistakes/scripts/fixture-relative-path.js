const path = require("node:path");

function isWindowsPath(value) {
  return /^[A-Za-z]:[\\/]/u.test(value) || value.startsWith("\\\\") || value.startsWith("//");
}

function normalizeWindowsPath(value) {
  const normalized = value.replace(/\//gu, "\\");
  if (normalized.startsWith("\\\\?\\UNC\\")) return `\\\\${normalized.slice(8)}`;
  const drive = normalized.slice(4);
  return normalized.startsWith("\\\\?\\") && /^[A-Za-z]:\\/u.test(drive) ? drive : normalized;
}

function fixtureRelativePath(file, fixture) {
  const windows = isWindowsPath(file) || isWindowsPath(fixture);
  const api = windows ? path.win32 : path.posix;
  const normalize = windows ? normalizeWindowsPath : (value) => value;
  const root = api.resolve(normalize(fixture));
  const normalizedFile = normalize(file);
  const absoluteFile = api.isAbsolute(normalizedFile)
    ? normalizedFile
    : api.resolve(root, normalizedFile);
  const relative = api.relative(root, absoluteFile);
  if (api.isAbsolute(relative) || relative === ".." || relative.startsWith(`..${api.sep}`)) {
    throw new Error(`${file} is outside or on a different root from ${fixture}`);
  }
  return relative.split(api.sep).join("/");
}

module.exports = { fixtureRelativePath };
