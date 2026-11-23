type JobOptions = {
  attempts?: number;
  removeOnComplete?: boolean;
  priority?: number;
  jobId?: string;
  deduplication?: { mode: string; ttl: number };
};

type Queue = { name: string };
type Enqueue = (data: unknown, options?: JobOptions) => Promise<void>;
type ObjectDefinition = {
  queue?: Queue;
  queueName: string;
  jobName: string;
  defaults: JobOptions;
};

export declare function createQueue(name: string): Queue;
export declare function createWorker(
  name: string,
  processor: (job: { name: string }) => Promise<void>,
): object;
export declare function makeObjectEnqueue(definition: ObjectDefinition): Enqueue;
export declare function makeObjectProducer(definition: ObjectDefinition): object;
export declare function makeReturnedEnqueue(queue: Queue, job: string): Enqueue;
export declare function makeFixedEnqueue(definition: { name: string }): Enqueue;
export declare function makeFixedBulkEnqueue(
  definition: { name: string },
  build: (row: unknown) => unknown,
): Enqueue;
