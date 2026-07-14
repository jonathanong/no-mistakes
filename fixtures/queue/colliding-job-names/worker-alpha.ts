import { Worker } from "bullmq";

export const alphaWorker = new Worker("alpha", async (job) => {
  if (job.name === "send") return job.data;
});
