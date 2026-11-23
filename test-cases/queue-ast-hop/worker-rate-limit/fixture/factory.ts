import { Queue, Worker } from "bullmq";

export function createQueue(name: string) {
  return new Queue(name);
}

export function createWorker(name: string, processor: () => Promise<void>) {
  return new Worker(name, processor);
}

export function getUnknownWorker(): { rateLimit(ms: number): Promise<void> } {
  return {
    async rateLimit(_ms: number) {},
  };
}

export class RateLimiter {
  async rateLimit(_ms: number) {}
}

export const sharedLimiter = new RateLimiter();
