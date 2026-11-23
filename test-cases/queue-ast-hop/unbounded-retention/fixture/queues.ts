import { Queue } from "bullmq";

declare const dynamicRetention: boolean | number;

// Reported: failed jobs are kept forever in defaultJobOptions.
export const unboundedDefault = new Queue("unboundedDefault", {
  defaultJobOptions: {
    attempts: 3,
    removeOnComplete: true,
    removeOnFail: false,
  },
});

// Accepted: both bounded by true / age + count.
export const bounded = new Queue("bounded", {
  defaultJobOptions: {
    attempts: 3,
    removeOnComplete: true,
    removeOnFail: { age: 86_400, count: 1_000 },
  },
});

// No retention defaults: producers must bound both.
export const bare = new Queue("bare", {
  defaultJobOptions: { attempts: 3 },
});

// Dynamic default value: abstains.
export const dynamicQueue = new Queue("dynamicQueue", {
  defaultJobOptions: {
    attempts: 3,
    removeOnComplete: true,
    removeOnFail: dynamicRetention,
  },
});
