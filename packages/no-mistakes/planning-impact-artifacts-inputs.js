"use strict";

const { realpath, stat } = require("node:fs/promises");
const { basename, dirname, join } = require("node:path");

const RESERVED_ARTIFACT_NAME =
  /^(?:dependencies|dependents|symbols|plan)\.(?:json|stderr|status)$/u;
const RESERVED_ARTIFACT_NAMES = ["dependencies", "dependents", "symbols", "plan"].flatMap((name) =>
  ["json", "stderr", "status"].map((extension) => `${name}.${extension}`),
);

async function isReservedArtifactPath(path) {
  if (RESERVED_ARTIFACT_NAME.test(basename(path))) return true;
  for (const name of RESERVED_ARTIFACT_NAMES) {
    try {
      if ((await realpath(join(dirname(path), name))) === path) return true;
    } catch (error) {
      if (!["ENOENT", "ENOTDIR"].includes(error.code)) throw error;
    }
  }
  return false;
}

async function existingFiles(root, files) {
  try {
    await stat(root);
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

module.exports = { buildRequest, completeResult, isReservedArtifactPath };
