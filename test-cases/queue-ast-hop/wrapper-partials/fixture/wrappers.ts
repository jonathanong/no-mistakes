type JobOptions = { attempts?: number; priority?: number };
type Job = { name: string };

export declare function createQueue(name: string): object;
export declare function createWorker(
  name: string,
  processor: (job: Job) => Promise<void>,
): object;
export declare function makePartial(
  definition: { name: string },
  defaults?: JobOptions,
): (data: unknown, options?: JobOptions) => Promise<void>;
export declare function makeOptions(
  job: string,
  envelope: { options?: JobOptions },
  defaults?: JobOptions,
): Promise<void>;
