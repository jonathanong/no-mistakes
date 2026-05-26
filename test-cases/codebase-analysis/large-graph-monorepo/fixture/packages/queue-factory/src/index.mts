export function createQueue(name: string, options?: unknown) {
  return {
    name,
    options,
    add(jobName: string, payload: unknown) {
      return Promise.resolve({ jobName, payload });
    },
    addBulk(jobs: Array<{ name: string; data: unknown }>) {
      return Promise.resolve(jobs);
    },
  };
}
