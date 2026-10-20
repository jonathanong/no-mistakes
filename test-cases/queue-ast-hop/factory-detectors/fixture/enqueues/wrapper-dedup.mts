import { createWorker, enqueueJob } from "../lib.mts";
import { queue } from "../queues.mts";

export const worker = createWorker("jobs", async (job) => {
  switch (job.name) {
    case "page":
      enqueueJob(queue, "page", {}, { deduplication: { id: job.id } });
      enqueueJob(queue, "page", {}, { deduplication: { mode: "debounce", ttl: 5 } });
  }
});
