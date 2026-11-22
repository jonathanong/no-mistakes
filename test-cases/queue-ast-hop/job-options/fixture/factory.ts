export function createQueue(name: string, _options?: object): unknown {
  return name;
}

export function enqueueWidget(..._args: unknown[]): void {}

export const foreignDefaults = {
  defaultJobOptions: { attempts: 1, removeOnComplete: true, backoff: 1 },
};
