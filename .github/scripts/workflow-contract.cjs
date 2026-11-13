function workflowJob(workflow, name) {
  const body = workflow.split(`\n  ${name}:\n`)[1]?.split(/\n  [a-z][a-z-]*:\n/)[0];
  if (!body) throw new Error(`workflow must retain the ${name} job`);
  return body;
}
module.exports = { workflowJob };
