import { Worker } from "bullmq";

export function startWorker(name: string, processor: () => Promise<void>) {
  return new Worker(name, processor);
}
