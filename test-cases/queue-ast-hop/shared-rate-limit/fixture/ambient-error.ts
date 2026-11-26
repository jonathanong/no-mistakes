import { Worker } from "bullmq";

export const ambientWorker = new Worker("widgets", async () => {
  throw new RateLimitError();
});
