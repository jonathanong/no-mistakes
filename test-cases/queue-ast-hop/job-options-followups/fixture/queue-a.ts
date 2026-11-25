import { Queue } from "bullmq";

export const sharedA = new Queue("shared", {
  defaultJobOptions: { attempts: 3 },
});
