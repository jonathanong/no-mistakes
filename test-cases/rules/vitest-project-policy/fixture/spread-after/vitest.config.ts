export default { test: { projects: [
  { test: { name: 'overridden', fileParallelism: false, maxWorkers: 1, ...dynamic } },
] } };
