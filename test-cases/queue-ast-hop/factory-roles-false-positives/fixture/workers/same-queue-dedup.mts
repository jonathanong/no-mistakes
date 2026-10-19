import { createWorker } from "../lib.mts";
import { a } from "../queues.mts";

export const wa = createWorker("a", async (job) => {
  switch (job.name) {
    case "x":
      await a.add("x", {}, { deduplication: { id: job.id } });
      await a.add("y", {}); a.addBulk("x", {}, { deduplication: { id: job.id } });
      await missing.add("x", {}, { jobId: job.id });
      return;
  }
});
