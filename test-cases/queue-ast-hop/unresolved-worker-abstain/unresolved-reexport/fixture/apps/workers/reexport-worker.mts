import { Worker } from "bullmq";
import { MISSING_QUEUE } from "./reexport.mts";

export const reexportWorker = new Worker(MISSING_QUEUE, async (job) => {
  if (job.name === "published") {
    return;
  }
});
