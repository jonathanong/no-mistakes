import { Queue } from "bullmq";

export const shared = new Queue("shared", {
  defaultJobOptions: { removeOnComplete: true },
});
