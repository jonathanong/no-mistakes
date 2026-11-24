import { RateLimiter, createQueue } from "./factory";
import * as bullmq from "bullmq";

class StaticLimiter {
  static async rateLimit(_ms: number) {}
}

export async function throttleNonWorkers() {
  const limiter = new RateLimiter();
  await limiter.rateLimit(1000);

  const inline = {
    async rateLimit(_ms: number) {},
  };
  await inline.rateLimit(1000);

  const queue = createQueue("scratch");
  await (queue as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(1000);

  await StaticLimiter.rateLimit(1000);
  await (bullmq as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(1000);

  function shadowed() {
    class Worker {
      async rateLimit(_ms: number) {}
    }
    const local = new Worker();
    return local.rateLimit(1000);
  }
  await shadowed();
}
