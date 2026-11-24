import { Worker } from "bullmq";

declare function resolveQueueName(): string;

export const dynamicWorker = new Worker(resolveQueueName(), async () => {});
