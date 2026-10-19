import { createWorker } from "../lib.mts";
import { b } from "../queues.mts";

export const wa = createWorker("a", async (job) => {
  switch (job.name) {
    case "x":
      await b.add("x", {}, { jobId: job.id });
      return;
  }
});
