#!/usr/bin/env node
"use strict";

const { spawn } = require("node:child_process");
const { join } = require("node:path");
const { boundedDiagnostic } = require("../planning-impact-cli");

const VALUE_OPTIONS = new Set(["--timeout", "--lock-timeout", "--jobs", "--profile", "-j"]);
const FLAG_OPTIONS = new Set(["--fail-on-lock"]);
const INLINE_VALUE_OPTION = /^(?:--(?:timeout|lock-timeout|jobs|profile)=|-j)/u;

function firstSubcommand(argv) {
  const index = firstSubcommandIndex(argv);
  return index === undefined ? undefined : argv[index];
}

function firstSubcommandIndex(argv) {
  for (let index = 0; index < argv.length; index += 1) {
    const value = argv[index];
    if (value === "--") return undefined;
    if (VALUE_OPTIONS.has(value)) {
      index += 1;
      continue;
    }
    if (FLAG_OPTIONS.has(value) || INLINE_VALUE_OPTION.test(value)) {
      continue;
    }
    return index;
  }
  return undefined;
}

function planningImpactArgs(argv) {
  const index = firstSubcommandIndex(argv);
  return [...argv.slice(0, index), ...argv.slice(index + 1)];
}

function launchNative(argv, spawnFn = spawn, io = process, kill = process.kill) {
  const child = spawnFn(join(__dirname, "no-mistakes"), argv, { stdio: "inherit" });
  child.on("error", (error) => {
    io.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
    io.exitCode = 1;
  });
  child.on("exit", (code, signal) => {
    if (signal) kill(process.pid, signal);
    else io.exitCode = code === null ? 1 : code;
  });
  return child;
}

async function main(
  argv = process.argv.slice(2),
  {
    io = process,
    launchNativeFn = launchNative,
    runPlanning = (args, planningIo) =>
      require("../planning-impact-cli").main(args, process.cwd(), planningIo),
  } = {},
) {
  if (firstSubcommand(argv) === "planning-impact") {
    try {
      await runPlanning(planningImpactArgs(argv), io);
    } catch (error) {
      io.stderr.write(boundedDiagnostic(error));
      io.exitCode = 1;
    }
  } else {
    launchNativeFn(argv);
  }
}

if (require.main === module) void main();

module.exports = { firstSubcommand, launchNative, main, planningImpactArgs };
