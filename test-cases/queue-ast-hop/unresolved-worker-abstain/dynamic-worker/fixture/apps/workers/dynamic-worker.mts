import { Worker } from "bullmq";
import { EVENTS_QUEUE } from "@acme/queues-events/config";

declare function withPrefix(queueName: string): string;

export const dynamicWorker = new Worker(withPrefix(EVENTS_QUEUE), async () => {});
