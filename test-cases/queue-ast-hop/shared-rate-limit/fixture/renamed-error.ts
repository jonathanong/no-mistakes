import { RateLimitError as Pause, Worker } from "bullmq";

export const worker = new Worker("widgets", async () => {
  throw new Pause();
});
