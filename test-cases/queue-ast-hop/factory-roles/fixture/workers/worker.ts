import { BACKGROUND_QUEUE_NAME } from "../config";
import { createWorker } from "../lib/factory";
import { background, other } from "../queues";

export const worker = createWorker(BACKGROUND_QUEUE_NAME, async (job) => {
  // Negative shapes sit outside the switch so they are judged with no case
  // label: a non-literal job, a missing options object, a dedup key that is
  // not the active job, and debounce options that are not `{ mode, ttl }`.
  await background.add(job.name, {});
  await background.add("page", {});
  await background.add("page", {}, { other: 1 });
  await background.add("page", {}, { jobId: job.id });
  await background.add("page", {}, { jobId: getJob().id });
  await background.add("page", {}, { deduplication: job.id });
  await background.add("page", {}, { deduplication: { ...{ id: job.id } } });
  await background.add("other", {}, "ttl");
  await background.add("other", {}, { ...{ deduplication: { mode: "debounce", ttl: 1 } } });
  await background.add("other", {}, { deduplication: "debounce" });
  await background.add("other", {}, { deduplication: { ...{ mode: "debounce" }, ttl: 1 } });
  switch (job.name) {
    case "page":
      await background.add("page", {}, { deduplication: { id: job.id } });
      await other.add("pong", {});
      return;
    case "stale":
      await background.add("other", {}, { deduplication: { mode: "debounce", ttl: 1000 } });
      return;
  }
});

export const functionWorker = createWorker("background", async function (job) {
  switch (job) {
    case "page":
      return;
  }
  switch (calls().name) {
    case "page":
      return;
  }
});
