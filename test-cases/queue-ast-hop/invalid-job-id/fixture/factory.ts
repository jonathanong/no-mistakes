import { Queue } from "bullmq";

export function enqueueConfigured(_queue: unknown, _job: string, _options: object): void {}

export function makeReviewQueue(name: string) {
  return new Queue(name);
}
