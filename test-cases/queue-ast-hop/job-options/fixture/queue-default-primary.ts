import { Queue } from "bullmq";

export const primary = new Queue("shared", {
  defaultJobOptions: { attempts: 1, removeOnComplete: true },
});
