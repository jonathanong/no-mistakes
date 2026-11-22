import { Queue } from "bullmq";

export function enqueueConfigured(_queue: unknown, _job: string, _options?: object): void {}

export function makeReviewQueue(name: string) {
  return new Queue(name);
}

export const registry = {
  add(_job: string, _data: object, _options: object) {},
  nested: { add(_job: string, _data: object, _options: object) {} },
};
