// fileParallelism: false
export const computed = {
  maxWorkers: WORKERS,
  maxWorkers: limits.workers,
  maxWorkers: parseVitestMaxWorkers(),
  fileParallelism: true,
  fileParallelism: FALSE_CONSTANT,
};
