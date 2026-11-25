function workflowJob(workflow, name) {
  const body = workflow.split(`\n  ${name}:\n`)[1]?.split(/\n  [a-z][a-z-]*:\n/)[0];
  if (!body) throw new Error(`workflow must retain the ${name} job`);
  return body;
}
function readWorkflow(file) {
  const { readFileSync } = require("node:fs");
  const { parse } = require("yaml");
  return parse(readFileSync(file, "utf8"));
}

function workflowGateCommand(job) {
  const run = job.match(/^ {8}run: node (\.github\/scripts\/[\w.-]+\.cjs)$/m)?.[1];
  if (run) {
    const path = require("node:path");
    return [process.execPath, [path.resolve(__dirname, "../..", run)]];
  }
  const script = job.split("        run: |\n")[1].replace(/^          /gm, "");
  return ["bash", ["-eu", "-c", script]];
}

module.exports = { workflowJob, readWorkflow, workflowGateCommand };
