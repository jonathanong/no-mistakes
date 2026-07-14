import { Worker } from "bullmq";

// Both typed jobs intentionally collapse to the same public edge identity.
export const alphaWorker = new Worker("alpha", async (job) => {
  if (job.name === "send") return job.data;
});
export const betaWorker = new Worker("beta", async (job) => {
  if (job.name === "send") return job.data;
});
