import { RateLimitError, Worker } from "bullmq";

export const worker = new Worker("drafts", async () => {
  throw new RateLimitError();
});
