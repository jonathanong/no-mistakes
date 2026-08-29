"use strict";

const {
  publishArtifact,
  removeArtifact,
  updateOutputDirectory,
  validateManifest,
  validateOutputDirectory,
} = require("./planning-impact-artifacts-files");
const artifactErrors = require("./planning-impact-artifacts-errors");
const { realpath } = require("node:fs/promises");
const { existingFiles } = require("./planning-impact-artifacts-inputs");
const { basename, isAbsolute, posix, resolve, win32 } = require("node:path");

const REPORTS = ["dependencies", "dependents", "symbols", "plan"];
const REPORT_TYPES = {
  dependencies: "dependencies",
  dependents: "dependents",
  symbols: "symbols",
  plan: "testsPlan",
};
const RESERVED_ARTIFACT_NAME =
  /^(?:dependencies|dependents|symbols|plan)\.(?:json|stderr|status)$/iu;

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
        { id: "symbols", type: "symbols", files: symbolFiles, include: "both" },
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

async function writePlanningImpactArtifacts(options, analyzeProject, renameNoReplace) {
  const output = await validateOutputDirectory(options.outputDirectory);
  let manifestHandle;
  let mayWriteFailureArtifacts = false;
  try {
    const requestedManifestPath = resolve(options.changedFilesManifest);
    const manifestPath = await realpath(requestedManifestPath);
    mayWriteFailureArtifacts = !RESERVED_ARTIFACT_NAME.test(basename(manifestPath));
    const manifest = await validateManifest(output, manifestPath);
    manifestHandle = manifest.handle;
    if (RESERVED_ARTIFACT_NAME.test(basename(manifest.path))) {
      mayWriteFailureArtifacts = false;
      throw new Error("changed-files manifest must not use a reserved artifact destination");
    }
    const changedFiles = parseChangedFiles(await manifestHandle.readFile("utf8"));
    await manifestHandle.close();
    manifestHandle = undefined;
    await updateOutputDirectory(output, invalidateStatuses, renameNoReplace);
    const request = {
      ...(await buildRequest(options.root, changedFiles, options.broad === true)),
      ...invocationOptions(options),
    };
    const result = await analyzeProject(request);
    const completed = completeResult(result, request.reports.length > 1);
    const artifacts = reportArtifacts(completed);
    await updateOutputDirectory(
      output,
      (privateOutput) => writeSuccess(privateOutput, artifacts),
      renameNoReplace,
    );
    return { outputDirectory: output.path, ...artifacts };
  } catch (error) {
    if (manifestHandle) {
      await manifestHandle.close().catch(() => {});
      manifestHandle = undefined;
    }
    if (mayWriteFailureArtifacts && !artifactErrors.isOutputRestorationFailure(error)) {
      try {
        await updateOutputDirectory(
          output,
          (privateOutput) => writeFailure(privateOutput, error),
          renameNoReplace,
        );
      } catch (failureReportingError) {
        if (artifactErrors.isOutputRestorationFailure(failureReportingError)) {
          throw artifactErrors.preserveFailureReportingError(error, failureReportingError);
        }
      }
    }
    throw error;
  } finally {
    if (manifestHandle) await manifestHandle.close().catch(() => {});
    if (output.handle) await output.handle.close().catch(() => {});
  }
}
function invocationOptions(options) {
  return Object.fromEntries(
    ["timeout", "lockTimeout", "failOnLock", "jobs", "profile"]
      .filter((name) => Object.hasOwn(options, name))
      .map((name) => [name, options[name]]),
  );
}
function parseChangedFiles(source) {
  const files = [...new Set(source.split(/\r\n|[\r\n]/u).filter((line) => line.length > 0))];
  if (!files.length) throw new Error("changed-files manifest is empty");
  for (const file of files) {
    if (
      isAbsolute(file) ||
      posix.isAbsolute(file) ||
      win32.isAbsolute(file) ||
      /^[A-Za-z]:/u.test(file) ||
      file.split(/[\\/]/u).includes("..")
    ) {
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

async function writeSuccess(output, artifacts) {
  for (const name of REPORTS) {
    await publishArtifact(
      output,
      `${name}.json`,
      `${JSON.stringify(toCliValue(artifacts[name]))}\n`,
    );
    await publishArtifact(output, `${name}.stderr`, "");
  }
  for (const name of REPORTS) await publishArtifact(output, `${name}.status`, "0\n");
}

async function writeFailure(output, error) {
  const diagnostic = boundedDiagnostic(error);
  for (const name of REPORTS) {
    await attempt(() => removeArtifact(output, `${name}.json`));
    await attempt(() => publishArtifact(output, `${name}.stderr`, diagnostic));
    await attempt(() => publishArtifact(output, `${name}.status`, "1\n"));
  }
}

async function invalidateStatuses(output) {
  for (const name of REPORTS) {
    await publishArtifact(output, `${name}.status`, "1\n");
  }
}

async function attempt(operation) {
  try {
    await operation();
  } catch {
    // Failure reporting must preserve the original analysis/publication error.
  }
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
