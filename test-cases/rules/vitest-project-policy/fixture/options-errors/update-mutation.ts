const project = { test: { fileParallelism: false, maxWorkers: 1 } };
project.test.maxWorkers++;
export default { test: { projects: [project] } };
