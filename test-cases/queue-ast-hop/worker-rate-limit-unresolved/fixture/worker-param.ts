import type { Worker } from "bullmq";

export async function pauseWorker(worker: Worker) {
  await worker.rateLimit(1000);
}
