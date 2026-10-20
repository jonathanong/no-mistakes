import { createWorker } from "../lib.mts";
import { queue } from "../queues.mts";

// The map callback's first parameter is not the processor, so `item.id` is not
// a self-dedup even though the processor's first parameter would be.
export const mapped = createWorker("jobs", async (task) => {
  const items = [{ id: task.id }];
  return items.map((item) => queue.add("jobs", item, { jobId: item.id }));
});

// A nested parameter that is literally named `job` is still the active job.
export const forwarded = createWorker("jobs", async (task) => {
  const send = (job: { name: string; id: string }) => {
    switch (job.name) {
      case "page":
        return queue.add("page", {}, { jobId: job.id });
    }
    return task;
  };
  return send(task);
});
