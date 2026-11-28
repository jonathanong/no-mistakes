import { createWorker, enqueueJob } from "../lib.mts";
import { queue } from "../queues.mts";

declare const unknownDedup: object;

export const worker = createWorker("jobs", async (job) => {
  switch (job.name) {
    case "page":
      enqueueJob(queue, "page", {}, { deduplication: { id: job.id } });
      enqueueJob(queue, "page", {}, { deduplication: { mode: "debounce", ttl: 5 } });
      enqueueJob(queue, "page", {}, { deduplication: { mode: job.name, ttl: 5 } });
      enqueueJob(queue, "page", {}, { ...unknownDedup });
  }
});
