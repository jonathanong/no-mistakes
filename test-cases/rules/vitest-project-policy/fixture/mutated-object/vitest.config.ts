const project = { test: { fileParallelism: false, maxWorkers: 1 } };
project.test.maxWorkers = 2;
export default { test: { projects: [project] } };
