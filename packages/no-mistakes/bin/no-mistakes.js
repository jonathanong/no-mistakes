#!/usr/bin/env node
"use strict";

const { spawn } = require("node:child_process");
const { join } = require("node:path");

function binaryPath(platform = process.platform) {
  const binName = platform === "win32" ? "no-mistakes.exe" : "no-mistakes";
  return join(__dirname, "..", "vendor", binName);
}

function run(
  argv = process.argv.slice(2),
  platform = process.platform,
  io = process,
  spawnFn = spawn,
) {
  const child = spawnFn(binaryPath(platform), argv, {
    stdio: "inherit",
  });

  child.on("exit", (code, signal) => {
    if (code !== null) {
      io.exit(code);
      return;
    }
    if (signal !== null) {
      io.exit(1);
      return;
    }
    io.exit(0);
  });

  child.on("error", (error) => {
    console.error(error);
    io.exit(1);
  });
}

if (require.main === module) run();

module.exports = { binaryPath, run };
