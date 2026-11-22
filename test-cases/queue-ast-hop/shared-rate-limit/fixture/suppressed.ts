import { RateLimitError, Worker } from "bullmq";

export const worker = new Worker("holds", async () => {
  // no-mistakes-disable-next-line queues-check
  throw new RateLimitError();
});
