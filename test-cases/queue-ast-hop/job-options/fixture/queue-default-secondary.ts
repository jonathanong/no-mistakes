import { Queue } from "bullmq";

export const secondary = new Queue("shared", {
  defaultJobOptions: { attempts: 1, backoff: 1 },
});
