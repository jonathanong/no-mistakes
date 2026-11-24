import { Queue } from "bullmq";
import { makeQueue, runtimeQueueName } from "./factory";

export const blank = new Queue("blank");
export const bounded = new Queue("bounded", {
  defaultJobOptions: { removeOnComplete: true, removeOnFail: { age: 3600, count: 20 } },
});
export const retained = new Queue("retained", {
  defaultJobOptions: { removeOnComplete: 0, removeOnFail: false },
});
export const partial = new Queue("partial", {
  defaultJobOptions: { removeOnComplete: { count: 10 } },
});

declare function getDefaults(): object;
export const dynamic = new Queue("dynamic", { defaultJobOptions: getDefaults() });

const config = { defaultJobOptions: { removeOnComplete: false, removeOnFail: true } };
export const factory = makeQueue("factory", { ...config });

export const shared = new Queue("shared", {
  defaultJobOptions: { removeOnComplete: true, removeOnFail: true },
});

declare const queueArgs: [string, object];
const shiftedConstructor = new Queue(...queueArgs, { defaultJobOptions: { removeOnFail: false } });

export const duplicateWithBounds = new Queue("duplicate", {
  defaultJobOptions: { removeOnComplete: true },
});
export const duplicateWithoutDefaults = new Queue("duplicate");
duplicateWithoutDefaults.add("conflicting-defaults", {});

export const runtimeQueue = makeQueue(runtimeQueueName, {
  defaultJobOptions: { removeOnFail: false },
});
