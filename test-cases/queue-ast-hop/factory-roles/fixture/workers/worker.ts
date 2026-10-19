import { BACKGROUND_QUEUE_NAME } from "../config";
import { createWorker } from "../lib/factory";
import { background } from "../queues";

export const worker = createWorker(BACKGROUND_QUEUE_NAME, async (job) => {
  switch (job.name) {
    case "page":
      await background.add("page", {}, { deduplication: { id: job.id } });
      return;
    case "stale":
      await background.add("other", {}, { deduplication: { mode: "debounce", ttl: 1000 } });
      return;
  }
});
