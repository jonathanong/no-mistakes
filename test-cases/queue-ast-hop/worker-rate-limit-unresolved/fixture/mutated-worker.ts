import { Queue, Worker } from "bullmq";

export const emails = new Queue("emails");

export async function pauseWorker(override?: Worker) {
  let worker = new Worker("emails", async () => {});
  if (override) worker = override;
  await worker.rateLimit(1000);
}
