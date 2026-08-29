"use strict";

const { chmod, readFile, realpath, rm, stat, writeFile } = require("node:fs/promises");
const { dirname, isAbsolute, join, resolve } = require("node:path");

const REPORTS = ["dependencies", "dependents", "symbols", "plan"];
const REPORT_TYPES = {
  dependencies: "dependencies",
  dependents: "dependents",
  symbols: "symbols",
  plan: "testsPlan",
};

function buildRequest(root, changedFiles, broad) {
  const structuralFiles = changedFiles.filter((file) => /\.[cm]?[jt]sx?$/u.test(file));
  const relationships = broad ? {} : { relationships: ["import", "workspace"] };
  const reports = structuralFiles.length
    ? [
        ...["dependencies", "dependents"].map((type) => ({
          id: type,
          type,
          files: structuralFiles,
          depth: 1,
          ...relationships,
        })),
        { id: "symbols", type: "symbols", files: structuralFiles, include: "both" },
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

async function writePlanningImpactArtifacts(options, analyzeProject) {
  const outputDirectory = await validateOutputDirectory(options.outputDirectory);
  const manifestPath = resolve(options.changedFilesManifest);
  try {
    await validateManifest(outputDirectory, manifestPath);
    const changedFiles = parseChangedFiles(await readFile(manifestPath, "utf8"));
    const request = {
      ...buildRequest(options.root, changedFiles, options.broad === true),
      ...invocationOptions(options),
    };
    const result = await analyzeProject(request);
    const completed = completeResult(result, request.reports.length > 1);
    const artifacts = reportArtifacts(completed);
    await writeSuccess(outputDirectory, artifacts);
    return { outputDirectory, ...artifacts };
  } catch (error) {
    await writeFailure(outputDirectory, error);
    throw error;
  }
}

function invocationOptions(options) {
  return Object.fromEntries(
    ["timeout", "lockTimeout", "failOnLock", "jobs", "profile"]
      .filter((name) => Object.hasOwn(options, name))
      .map((name) => [name, options[name]]),
  );
}

async function validateOutputDirectory(outputDirectory) {
  const directory = await realpath(outputDirectory);
  const metadata = await stat(directory);
  if (!metadata.isDirectory() || (metadata.mode & 0o077) !== 0) {
    throw new Error("output directory must exist and have mode 0700");
  }
  return directory;
}

async function validateManifest(outputDirectory, manifestPath) {
  if ((await realpath(dirname(manifestPath))) !== outputDirectory) {
    throw new Error("manifest must be inside the private output directory");
  }
  const manifest = await realpath(manifestPath);
  if (dirname(manifest) !== outputDirectory) {
    throw new Error("manifest must be inside the private output directory");
  }
}

function parseChangedFiles(source) {
  const files = [
    ...new Set(
      source
        .split(/\r?\n/u)
        .map((line) => line.trim())
        .filter(Boolean),
    ),
  ];
  if (!files.length) throw new Error("changed-files manifest is empty");
  for (const file of files) {
    if (isAbsolute(file) || file.split(/[\\/]/u).includes("..")) {
      throw new Error(`changed file must be repository-relative: ${file}`);
    }
  }
  return files;
}

function completeResult(result, hasStructuralReports) {
  if (hasStructuralReports) return result;
  const traversal = { roots: [], files: [], diagnostics: [], tsconfig_provenance: [] };
  return {
    reports: [
      { id: "dependencies", type: "dependencies", result: traversal },
      { id: "dependents", type: "dependents", result: traversal },
      { id: "symbols", type: "symbols", result: { roots: [], files: [] } },
      ...result.reports,
    ],
  };
}

function reportArtifacts(result) {
  const reports = new Map(result.reports.map((report) => [report.id, report]));
  return Object.fromEntries(
    REPORTS.map((name) => {
      const report = reports.get(name);
      if (!report) throw new Error(`no-mistakes omitted the ${name} report`);
      if (report.type !== REPORT_TYPES[name]) {
        throw new Error(`${name} type ${report.type}; expected ${REPORT_TYPES[name]}`);
      }
      return [name, report.result];
    }),
  );
}

async function writeSuccess(outputDirectory, artifacts) {
  await Promise.all(
    REPORTS.flatMap((name) => [
      writeArtifact(
        join(outputDirectory, `${name}.json`),
        `${JSON.stringify(toCliValue(artifacts[name]))}\n`,
      ),
      writeArtifact(join(outputDirectory, `${name}.stderr`), ""),
      writeArtifact(join(outputDirectory, `${name}.status`), "0\n"),
    ]),
  );
}

async function writeFailure(outputDirectory, error) {
  const diagnostic = boundedDiagnostic(error);
  await Promise.all(
    REPORTS.flatMap((name) => [
      rm(join(outputDirectory, `${name}.json`), { force: true }),
      writeArtifact(join(outputDirectory, `${name}.stderr`), diagnostic),
      writeArtifact(join(outputDirectory, `${name}.status`), "1\n"),
    ]),
  );
}

async function writeArtifact(path, contents) {
  await writeFile(path, contents, { mode: 0o600 });
  await chmod(path, 0o600);
}

function toCliValue(value) {
  if (Array.isArray(value)) return value.map(toCliValue);
  if (value === null || typeof value !== "object") return value;
  return Object.fromEntries(
    Object.entries(value).map(([key, child]) => [
      key.replace(/[A-Z]/gu, (letter) => `_${letter.toLowerCase()}`),
      toCliValue(child),
    ]),
  );
}

function boundedDiagnostic(error) {
  const detail = error instanceof Error ? `${error.name}: ${error.message}` : String(error);
  return Buffer.from(`${detail}\n`)
    .subarray(0, 4096)
    .toString("utf8")
    .replace(/\ufffd+$/u, "");
}

module.exports = { writePlanningImpactArtifacts };
