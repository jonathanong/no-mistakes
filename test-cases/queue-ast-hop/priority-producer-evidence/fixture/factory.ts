import { Queue } from "bullmq";

export function createQueue(name: string) {
  return new Queue(name);
}
