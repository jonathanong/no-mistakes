import { Queue } from "bullmq";

export const secondary = new Queue("shared", {
  defaultJobOptions: { backoff: 1 },
});
