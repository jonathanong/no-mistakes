import { Queue } from "bullmq";

const queue = new Queue("receipts", {
  defaultJobOptions: { removeOnComplete: true, removeOnFail: 10 },
});

declare const dynamicRetention: boolean;
declare const dynamicOptions: Record<string, unknown>;
declare const dynamicKey: string;

await queue.add("unknown-value", {}, { removeOnFail: dynamicRetention });
await queue.add("open-spread", {}, { ...dynamicOptions });
await queue.add("open-key", {}, { [dynamicKey]: true });
await queue.add("mixed", {}, { removeOnComplete: false, removeOnFail: dynamicRetention });
await queue.add("bounded", {}, { removeOnFail: 10 });
await queue.add("overrides-spread", {}, { ...dynamicOptions, removeOnComplete: true, removeOnFail: 10 });
await queue.add("unbounded", {}, { removeOnFail: false });
// no-mistakes-disable-next-line queues-check: Retention comes from the migration environment.
await queue.add("suppressed", {}, { removeOnFail: dynamicRetention });

declare const dynamicName: string;
const dynamicQueue = new Queue(dynamicName);
await dynamicQueue.add("dynamic-queue", {}, { removeOnComplete: true, removeOnFail: dynamicRetention });
await dynamicQueue.add("bounded-dynamic-queue", {}, { removeOnComplete: true, removeOnFail: 10 });

await dynamicQueue.add("dynamic-defaults", {}, { removeOnFail: 10 });
await dynamicQueue.add("unbounded-dynamic-queue", {}, { removeOnComplete: false, removeOnFail: 100 });

declare const collector: { add(name: string, data: object, options: object): void };
collector.add("unrelated", {}, { removeOnFail: dynamicRetention });
