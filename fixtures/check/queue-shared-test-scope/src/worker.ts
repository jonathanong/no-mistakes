import { Worker } from "bullmq";

export const ordersWorker = new Worker("orders", async (job) => {
  if (job.name === "create") {
    return job.id;
  }
  return null;
});
