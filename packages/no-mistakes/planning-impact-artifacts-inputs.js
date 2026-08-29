"use strict";

const { lstat, stat } = require("node:fs/promises");
const { join } = require("node:path");

async function existingFiles(root, files) {
  try {
    await lstat(root);
  } catch (error) {
    if (error.code === "ENOENT") return files;
    throw error;
  }
  const existing = await Promise.all(
    files.map(async (file) => {
      try {
        return (await stat(join(root, file))).isFile() ? file : undefined;
      } catch (error) {
        if (["ENOENT", "ENOTDIR"].includes(error.code)) return undefined;
        throw error;
      }
    }),
  );
  return existing.filter((file) => file !== undefined);
}

module.exports = { existingFiles };
