import { RateLimitError as Pause, Worker } from "bullmq";
import { orders } from "./queues";

class RateLimitError extends Error {}
const helper = { RateLimitError };

export const worker = new Worker("widgets", async () => {
  await orders.rateLimit(1000);
  {
    const worker = { async rateLimit(_ms: number) {} };
    await worker.rateLimit(1000);
  }
  if (Math.random()) throw new RateLimitError();
  if (Math.random()) throw new helper.RateLimitError();
  const Pause = Error;
  throw new Pause();
});
