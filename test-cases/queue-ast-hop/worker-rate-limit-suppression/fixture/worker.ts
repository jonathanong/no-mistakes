import { Queue, Worker } from "bullmq";

export const parcels = new Queue("parcels");

export async function pauseWorker(override?: Worker) {
  let worker = new Worker("parcels", async () => {});
  if (override) worker = override;
  await worker.rateLimit(1000);
  // no-mistakes-disable-next-line queues-check
  await worker.rateLimit(2000);
  await worker.rateLimit(3000); // no-mistakes-disable-line queues-check
}
