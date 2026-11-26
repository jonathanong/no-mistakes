import type { Job, Worker } from "bullmq";

export declare function createWorker(
  name: string,
  handler: (job: Job, worker: Worker) => Promise<void>,
): Worker;
