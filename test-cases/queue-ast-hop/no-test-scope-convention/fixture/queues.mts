import { Queue } from "bullmq";

export const ALERTS_QUEUE = "alerts";
export const SMOKE_QUEUE = "smoke";
export const alertsQueue = new Queue(ALERTS_QUEUE);
export const smokeQueue = new Queue(SMOKE_QUEUE);

export function createQueue(name: string) {
  return new Queue(name);
}

export function createWorker(_name: string, _processor: (job: { name: string }) => Promise<void>) {}
