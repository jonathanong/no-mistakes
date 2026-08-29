"use strict";

function isPrivateDirectory(metadata, platform = process.platform) {
  return metadata.isDirectory() && hasPrivateMode(metadata, 0o700, platform);
}

function isPrivateRegularFile(metadata, platform = process.platform) {
  return metadata.isFile() && hasPrivateMode(metadata, 0o600, platform);
}

function hasPrivateMode(metadata, mode, platform) {
  return platform !== "win32" && (metadata.mode & 0o7777) === mode;
}

function privatePermissionError(subject, mode, platform = process.platform) {
  if (platform === "win32") {
    return new Error(
      "planning impact artifacts are unavailable on Windows because Node cannot verify private Windows ACLs",
    );
  }
  return new Error(`${subject} must have mode ${mode.toString(8).padStart(4, "0")}`);
}

module.exports = { isPrivateDirectory, isPrivateRegularFile, privatePermissionError };
