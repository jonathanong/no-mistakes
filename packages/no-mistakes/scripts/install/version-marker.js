"use strict";

const { readFileSync } = require("node:fs");

function versionMarkerPath(destination) {
  return `${destination}.version`;
}

// The destination binary alone can't tell us which version it is (the CLI's
// own --version requires spawning a process, and the N-API .node addon isn't
// executable at all), so track the installed version in a small sidecar file
// written right after a successful install. A missing or mismatched marker
// means we can't prove the existing file matches the requested version, so
// checkExisting must fall through to a real (re)download rather than trust
// stale bytes left behind by an earlier install of a different version.
function installedVersion(destination) {
  try {
    return readFileSync(versionMarkerPath(destination), "utf8").trim();
  } catch (error) {
    if (error.code === "ENOENT") {
      return null;
    }
    throw error;
  }
}

module.exports = {
  installedVersion,
  versionMarkerPath,
};
