export function createQueue(name: string) {
  return {
    name,
    process: (fn: (job: { data: unknown }) => Promise<void>) => fn,
    add: async (name: string, data: unknown) => data,
  };
}
