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

async function buildRequest(root, changedFiles, broad) {
  const structuralFiles = changedFiles.filter((file) => /\.[cm]?[jt]sx?(?:#.*)?$/u.test(file));
  const traversalFiles = structuralFiles.map((file) => ({ file }));
  const symbolFiles = await existingFiles(root, structuralFiles);
  const relationships = broad ? {} : { relationships: ["import", "workspace"] };
  const reports = structuralFiles.length
    ? [
        ...["dependencies", "dependents"].map((type) => ({
          id: type,
          type,
          files: traversalFiles,
          depth: 1,
          ...relationships,
        })),
        ...(symbolFiles.length
          ? [{ id: "symbols", type: "symbols", files: symbolFiles, include: "both" }]
          : []),
      ]
    : [];
  reports.push({
    id: "plan",
    type: "testsPlan",
    framework: "vitest",
    environment: "prePush",
    changedFiles,
  });
  return { root, reports };
}

function completeResult(result, requestedReports) {
  const requested = new Set(requestedReports.map((report) => report.id));
  const traversal = { roots: [], files: [], diagnostics: [], tsconfig_provenance: [] };
  const omitted = [
    { id: "dependencies", type: "dependencies", result: traversal },
    { id: "dependents", type: "dependents", result: traversal },
    { id: "symbols", type: "symbols", result: { roots: [], files: [] } },
  ].filter((report) => !requested.has(report.id));
  if (!omitted.length) return result;
  return { reports: [...omitted, ...result.reports] };
}

module.exports = { buildRequest, completeResult };
