export default { test: { projects: [
  { test: { name: 'baseline', fileParallelism: false, maxWorkers: 1 } },
  { test: { name: 'drift', fileParallelism: true, maxWorkers: 2 } },
] } };
