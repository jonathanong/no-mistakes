"use strict";

const fs = require("node:fs");
const path = require("node:path");

const PACKAGE_NAME = /^(@[^/]*\/)?[^/]*/;
const EXTENSION = /(\.d)?\.[cm]?[jt]sx?$/;

function bare(file) {
  return path.resolve(file).replace(EXTENSION, "");
}

function packageNamed(file, name) {
  for (let dir = path.dirname(file); dir !== path.dirname(dir); dir = path.dirname(dir)) {
    let manifest;
    try {
      manifest = fs.readFileSync(path.join(dir, "package.json"), "utf8");
    } catch {
      continue;
    }
    try {
      return JSON.parse(manifest).name === name;
    } catch {
      return false;
    }
  }
}

// Whether a type is declared inside the configured module itself, as opposed to
// imported from it; see "Library executor terminals" in
// docs/eslint-rules/postgres-no-executor-adapter.md. `from` is the file that
// refers to the type, which resolves a relative `importSpecifier`.
function declaredInConfiguredModule(declarationFile, from, options) {
  const specifier = options.importSpecifier;
  if (specifier.startsWith(".")) {
    return bare(declarationFile) === bare(path.resolve(path.dirname(from), specifier));
  }
  return packageNamed(declarationFile, specifier.match(PACKAGE_NAME)[0]);
}

module.exports = { declaredInConfiguredModule };
