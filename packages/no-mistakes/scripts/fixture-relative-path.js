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

function fixtureRootAliases(fixture) {
  const windows = isWindowsPath(fixture);
  const api = windows ? path.win32 : path.posix;
  const root = api.resolve(windows ? normalizeWindowsPath(fixture) : fixture);
  const aliases = new Set([root, root.replace(/\\/gu, "/")]);
  if (windows) {
    const namespaced = path.win32.toNamespacedPath(root);
    aliases.add(namespaced);
    aliases.add(namespaced.replace(/\\/gu, "/"));
  }
  return [...aliases].sort((left, right) => right.length - left.length);
}

function normalizeFixtureReport(report, fixture) {
  const windows = isWindowsPath(fixture);
  const aliases = fixtureRootAliases(fixture).map((alias) => {
    const escaped = [...alias]
      .map((character) =>
        /[\\/]/u.test(character) ? "[\\\\/]" : character.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&"),
      )
      .join("");
    return new RegExp(`${escaped}(?:(?<separator>[\\\\/])|$)`, windows ? "giu" : "gu");
  });
  const normalize = (value) =>
    aliases.reduce(
      (result, alias) =>
        result.replace(alias, (_match, separator) => `<fixture>${separator ? "/" : ""}`),
      value,
    );
  const visit = (value) => {
    if (typeof value === "string") return normalize(value);
    if (Array.isArray(value)) return value.map(visit);
    if (value && typeof value === "object") {
      return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, visit(item)]));
    }
    return value;
  };
  return visit(report);
}

module.exports = { fixtureRelativePath, normalizeFixtureReport };
