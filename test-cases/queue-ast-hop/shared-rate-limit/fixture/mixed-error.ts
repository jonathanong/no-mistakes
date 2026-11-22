import { RateLimitError, Worker } from "bullmq";

export const worker = new Worker("widgets", async () => {
  throw new RateLimitError();
  throw Worker.RateLimitError();
});
