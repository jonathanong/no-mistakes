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

module.exports = { workflowJob, readWorkflow };
