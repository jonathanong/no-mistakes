export default { test: {
  exclude: [],
  projects: [
    { test: { name: 'widgets', fileParallelism: false, maxWorkers: 1 } },
    { test: { name: 'items', fileParallelism: false, maxWorkers: 1 } },
  ],
} };
