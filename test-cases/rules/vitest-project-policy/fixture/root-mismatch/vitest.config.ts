export default { test: { fileParallelism: false, maxWorkers: 1, projects: [
  { test: { name: 'root-drift', fileParallelism: true, maxWorkers: 2 } },
] } };
