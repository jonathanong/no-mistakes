// no-mistakes-disable-file queues-check: unmatched topology is intentional
import { createWorker } from "../lib.mts";
import { queue } from "../queues.mts";

export const worker = createWorker("jobs", async (job) => {
  switch (job.name) {
    case "page":
      await queue.add("page", {}, { deduplication: { id: job.id } });
  }
});
