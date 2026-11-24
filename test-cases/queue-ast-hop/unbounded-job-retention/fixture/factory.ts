import { Queue } from "bullmq";

export function makeQueue(name: string, options?: object) {
  return new Queue(name, options);
}

export function submitJob(queue: Queue, name: string, data: object, options: object) {
  return queue.add(name, data, options);
}
