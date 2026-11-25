import { Worker } from "bullmq";

export const gammaWorker = new Worker("gamma", async (job) => {
  if (job.name === "sync") {
    return;
  }
});
