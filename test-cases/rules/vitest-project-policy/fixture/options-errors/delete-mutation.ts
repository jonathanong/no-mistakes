const project = { test: { fileParallelism: false, maxWorkers: 1 } };
delete project.test.maxWorkers;
export default { test: { projects: [project] } };
