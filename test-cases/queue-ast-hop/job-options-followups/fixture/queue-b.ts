import { Queue } from "bullmq";

export const sharedB = new Queue("shared", {
  defaultJobOptions: { backoff: 1000 },
});
