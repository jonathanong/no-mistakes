const transform = (value) => ({ test: {} });
export default transform({ test: { projects: [{ test: { fileParallelism: false, maxWorkers: 1 } }] } });
