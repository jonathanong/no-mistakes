"use strict";

const { parseArgs } = require("node:util");

const USAGE = [
  "usage: no-mistakes planning-impact --changed-files <manifest>",
  "--output-dir <directory> [--broad]",
].join(" ");

function parseNonNegativeInteger(name, value) {
  if (!/^(?:0|[1-9][0-9]*)$/u.test(value)) {
    throw new Error(`${name} must be a non-negative integer`);
  }
  const parsed = Number(value);
  if (!Number.isSafeInteger(parsed)) throw new Error(`${name} must be a safe integer`);
  return parsed;
}

function parsePlanningImpactArgs(argv, root) {
  const { values } = parseArgs({
    args: argv,
    options: {
      "changed-files": { type: "string" },
      "output-dir": { type: "string" },
      broad: { type: "boolean" },
      timeout: { type: "string" },
      "lock-timeout": { type: "string" },
      "fail-on-lock": { type: "boolean" },
      jobs: { type: "string", short: "j" },
      profile: { type: "string" },
      help: { type: "boolean", short: "h" },
    },
    strict: true,
  });
  if (values.help) return null;
  if (!values["changed-files"] || !values["output-dir"]) throw new Error(USAGE);
  if (values.profile !== undefined && values.profile !== "ci") {
    throw new Error('profile must be "ci" when set');
  }
  const options = {
    root,
    changedFilesManifest: values["changed-files"],
    outputDirectory: values["output-dir"],
    broad: values.broad === true,
  };
  for (const [flag, key] of [
    ["timeout", "timeout"],
    ["lock-timeout", "lockTimeout"],
    ["jobs", "jobs"],
  ]) {
    if (values[flag] !== undefined) options[key] = parseNonNegativeInteger(flag, values[flag]);
  }
  if (values["fail-on-lock"]) options.failOnLock = true;
  if (values.profile) options.profile = values.profile;
  return options;
}

async function runPlanningImpact(argv, root, writer) {
  const options = parsePlanningImpactArgs(argv, root);
  if (options) await writer(options);
  return options;
}

function boundedDiagnostic(error) {
  const detail = error instanceof Error ? `${error.name}: ${error.message}` : String(error);
  return Buffer.from(`${detail}\n`)
    .subarray(0, 4096)
    .toString("utf8")
    .replace(/\ufffd+$/u, "");
}

async function main(argv = process.argv.slice(2), root = process.cwd(), io = process, writer) {
  const options = parsePlanningImpactArgs(argv, root);
  if (!options) {
    io.stdout.write(`${USAGE}\n`);
    return;
  }
  await (writer || require("./index").writePlanningImpactArtifacts)(options);
}

module.exports = { USAGE, boundedDiagnostic, main, parsePlanningImpactArgs, runPlanningImpact };
