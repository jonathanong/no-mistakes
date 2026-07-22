import { Worker } from "bullmq";

export const worker = new Worker("a-queue", async (job) => job.data);
