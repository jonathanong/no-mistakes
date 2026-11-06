let workers = 1;
const parallelism = false;
export default { test: { projects: [{ test: { fileParallelism: parallelism, maxWorkers: workers } }] } };
