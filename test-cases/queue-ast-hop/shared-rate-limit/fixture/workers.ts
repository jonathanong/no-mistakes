import type { Worker as QueueWorker } from "bullmq";
import type * as queue from "bullmq";
import { createWorker } from "./worker-factory";

export const worker = createWorker("items", async (_task, handle: QueueWorker) => {
  await handle.rateLimit(1000);
  await (async (handle: QueueWorker) => handle.rateLimit(1000))(handle);
});

interface Limiter {
  rateLimit(ms: number): Promise<void>;
}

export const unrelated = createWorker("items", async (job: queue.Job, helper: Limiter) => {
  const readName = ({ name }: QueueWorker) => name;
  if (job.name) await helper.rateLimit(1000);
  void readName;
});
