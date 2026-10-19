"use strict";

const path = require("node:path");

function checkExitCode(report) {
  const domains = ["react", "queues", "rules", "integration", "codebase", "warnings"];
  for (const domain of domains) {
    if (!Array.isArray(report[domain])) throw new Error(`missing check report array: ${domain}`);
  }
  return domains.some((domain) => report[domain].length > 0) ? 1 : 0;
}

async function run(check, root, jobs) {
  if (!["1", "max"].includes(jobs)) throw new Error("jobs must be 1 or max");
  const options = { root: path.resolve(root), timeout: 0, lockTimeout: 0 };
  if (jobs !== "max") options.jobs = Number(jobs);
  const report = await check(options);
  const exit = checkExitCode(report);
  process.stdout.write(`${JSON.stringify(report)}\n`);
  return exit;
}

if (require.main === module) {
  Promise.resolve()
    .then(async () => {
      if (!process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH)
        throw new Error("explicit measured addon required");
      const { check } = require("../../packages/no-mistakes");
      process.exitCode = await run(check, process.argv[2], process.argv[3]);
    })
    .catch((error) => {
      process.stderr.write(`${error.stack || error}\n`);
      process.exitCode = 2;
    });
}

module.exports = { checkExitCode, run };
