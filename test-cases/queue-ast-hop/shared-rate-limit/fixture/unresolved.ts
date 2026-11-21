import { RateLimitError, Worker } from "bullmq";

export function start(queueName: string) {
  return new Worker(queueName, async () => {
    throw new RateLimitError();
  });
}
