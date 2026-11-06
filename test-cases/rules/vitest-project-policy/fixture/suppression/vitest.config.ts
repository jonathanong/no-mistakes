// no-mistakes-disable-file vitest-project-policy
export default { test: { projects: [
  { test: { name: 'suppressed', fileParallelism: false } },
] } };
