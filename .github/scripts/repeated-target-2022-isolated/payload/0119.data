export function createQueue(name: string) {
  return {
    name,
    add: async (job: string, data: unknown) => ({ job, data }),
  };
}
