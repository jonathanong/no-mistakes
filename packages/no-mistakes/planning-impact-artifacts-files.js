"use strict";

const { lstat, open, realpath, rename, rm, stat } = require("node:fs/promises");
const { constants } = require("node:fs");
const { dirname, join } = require("node:path");
const { randomUUID } = require("node:crypto");

const MANIFEST_OPEN_FLAGS = constants.O_RDONLY | (constants.O_NOFOLLOW ?? 0);

async function validateOutputDirectory(outputDirectory) {
  const directory = await realpath(outputDirectory);
  const metadata = await stat(directory);
  if (!isPrivateDirectory(metadata)) {
    throw new Error("output directory must exist and have mode 0700");
  }
  let handle;
  try {
    handle = await open(directory, "r");
  } catch (error) {
    if (!["EISDIR", "EPERM"].includes(error.code)) throw error;
  }
  const output = { path: directory, identity: metadata, handle };
  try {
    await assertOutputDirectory(output);
    return output;
  } catch (error) {
    if (handle) await handle.close().catch(() => {});
    throw error;
  }
}

async function validateManifest(output, manifestPath) {
  await assertOutputDirectory(output);
  if ((await realpath(dirname(manifestPath))) !== output.path) {
    throw new Error("manifest must be inside the private output directory");
  }
  const manifest = await realpath(manifestPath);
  if (dirname(manifest) !== output.path) {
    throw new Error("manifest must be inside the private output directory");
  }
  const handle = await open(manifest, MANIFEST_OPEN_FLAGS);
  try {
    const metadata = await handle.stat();
    if (!metadata.isFile()) throw new Error("changed-files manifest must be a regular file");
    const pathMetadata = await lstat(manifest);
    if (!pathMetadata.isFile() || !sameFileIdentity(metadata, pathMetadata)) {
      throw new Error("changed-files manifest changed during validation");
    }
    await assertOutputDirectory(output);
    return { path: manifest, handle };
  } catch (error) {
    await handle.close().catch(() => {});
    throw error;
  }
}

async function publishArtifact(output, name, contents) {
  await assertOutputDirectory(output);
  const staged = join(output.path, `.${name}.${randomUUID()}.tmp`);
  const destination = join(output.path, name);
  let file;
  let identity;
  try {
    file = await open(staged, "wx", 0o600);
    try {
      await file.chmod(0o600);
      await file.writeFile(contents);
      identity = await file.stat();
      if (!identity.isFile() || (identity.mode & 0o7777) !== 0o600 || identity.nlink !== 1) {
        throw new Error(`staged artifact is not a private regular file: ${name}`);
      }
    } finally {
      await file.close();
      file = undefined;
    }
    const stagedMetadata = await lstat(staged);
    if (!sameFileIdentity(identity, stagedMetadata) || stagedMetadata.nlink !== 1) {
      throw new Error(`staged artifact changed before publication: ${name}`);
    }
    await assertOutputDirectory(output);
    await rename(staged, destination);
    await assertOutputDirectory(output);
    const published = await lstat(destination);
    if (
      !sameFileIdentity(identity, published) ||
      !published.isFile() ||
      (published.mode & 0o7777) !== 0o600 ||
      published.nlink !== 1
    ) {
      await removePath(output, destination).catch(() => {});
      throw new Error(`published artifact changed during publication: ${name}`);
    }
  } catch (error) {
    if (file) await file.close().catch(() => {});
    await removePath(output, staged).catch(() => {});
    throw error;
  }
}

async function removeArtifact(output, name) {
  return removePath(output, join(output.path, name));
}

async function removePath(output, target) {
  await assertOutputDirectory(output);
  let metadata;
  try {
    metadata = await lstat(target);
  } catch (error) {
    if (error.code === "ENOENT") return;
    throw error;
  }
  if (metadata.isDirectory()) return;
  await rm(target, { force: true });
  await assertOutputDirectory(output);
}

function isPrivateDirectory(metadata) {
  return metadata.isDirectory() && (metadata.mode & 0o7777) === 0o700;
}

async function assertOutputDirectory(output) {
  if ((await realpath(output.path)) !== output.path) {
    throw new Error("output directory path changed during planning artifact generation");
  }
  const metadata = await stat(output.path);
  if (!isPrivateDirectory(metadata) || !sameFileIdentity(output.identity, metadata)) {
    throw new Error("output directory changed during planning artifact generation");
  }
  if (output.handle) {
    const descriptorMetadata = await output.handle.stat();
    if (
      !isPrivateDirectory(descriptorMetadata) ||
      !sameFileIdentity(output.identity, descriptorMetadata)
    ) {
      throw new Error("output directory descriptor changed during planning artifact generation");
    }
  }
}

function sameFileIdentity(left, right) {
  return left.dev === right.dev && left.ino === right.ino;
}

module.exports = {
  assertOutputDirectory,
  publishArtifact,
  removeArtifact,
  validateManifest,
  validateOutputDirectory,
};
