import { Worker } from "bullmq";

export const betaWorker = new Worker("beta", async (job) => {
  if (job.name === "send") return job.data;
});
