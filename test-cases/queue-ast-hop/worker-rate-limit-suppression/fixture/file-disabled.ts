// no-mistakes-disable-file queues-check
import { Worker } from "bullmq";

export async function pauseWorker(override?: Worker) {
  let worker = new Worker("parcels", async () => {});
  if (override) worker = override;
  await worker.rateLimit(1000);
}
