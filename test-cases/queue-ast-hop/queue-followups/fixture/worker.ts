import { Worker } from "bullmq";

export const opaqueWorker = new Worker("opaque", async (job) => {
  if (job.name === "welcome") {
    return;
  }
});
export const bareWorker = new Worker("bare", async (job) => {
  if (job.name === "known-missing") {
    return;
  }
});
export const trackedWorker = new Worker("tracked", async (job) => {
  switch (job.name) {
    case "retry":
    case "audit":
      return;
  }
});
