// no-mistakes-disable-file vitest-project-policy
export default { test: { projects: [
  { test: { name: 'dynamic', fileParallelism: false, maxWorkers: process.env.WORKERS } },
] } };
