import { Queue } from "bullmq";

export const primary = new Queue("shared", {
  defaultJobOptions: { attempts: 1 },
});
primary.upsertJobScheduler("local-schedule", {}, { name: "real-local-scheduler" });

export const near = new Queue("local", {
  defaultJobOptions: { attempts: 1 },
});
export const far = new Queue("local", {
  defaultJobOptions: { backoff: 1 },
});
