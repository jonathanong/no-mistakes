"use strict";

const { repoRelativeFilename, stringMatches } = require("./module-mock-helpers");

// Modules whose `createRequire` export is Node's real createRequire()
// regardless of how a config bans/allows their other exports; tracked
// unconditionally (see no-banned-import-outside-allowed-paths-tags.js's
// `resolveNodeModuleCreateRequireTag`) since createRequire is itself a
// capability that can synchronously load any other banned module, whether
// obtained via a static `import` or a `require()` call.
const CREATE_REQUIRE_MODULES = new Set(["node:module", "module"]);

// Normalizes the `bannedImports` option into `Map<module, Set<name>>`. The
// sentinel name `"default"` bans the module's default export when it is used
// as a directly callable value; any other name bans that named export (or,
// when reached as a member off a namespace/default/require binding, that
// method/property name).
// `entries` is already schema-validated (each item requires a string `module`
// and a non-empty string-array `names`), so no defensive shape-checking here.
function normalizeBannedImports(entries) {
  const config = new Map();
  for (const entry of entries ?? []) {
    const names = config.get(entry.module) ?? new Set();
    for (const name of entry.names) names.add(name);
    config.set(entry.module, names);
  }
  return config;
}

function hasBannedName(config, module, name) {
  return config.get(module)?.has(name) ?? false;
}

function hasAnyBannedName(config, module) {
  return Boolean(config.get(module)?.size);
}

// `export * from "mod"` never re-exports a module's default export (ES module
// semantics), so the reserved "default" name must not by itself make an
// unaliased export-star declaration reachable.
function hasAnyNonDefaultBannedName(config, module) {
  const names = config.get(module);
  if (!names) return false;
  for (const name of names) {
    if (name !== "default") return true;
  }
  return false;
}

function shouldCheckFile(filename, options) {
  const checked = options?.checkedPathPatterns ?? [];
  if (checked.length === 0) return false;
  const file = repoRelativeFilename(filename);
  return stringMatches(file, checked) && !stringMatches(file, options.allowedPathPatterns ?? []);
}

module.exports = {
  CREATE_REQUIRE_MODULES,
  hasAnyBannedName,
  hasAnyNonDefaultBannedName,
  hasBannedName,
  normalizeBannedImports,
  shouldCheckFile,
};
