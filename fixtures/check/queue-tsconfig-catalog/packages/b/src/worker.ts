import { Worker } from "bullmq";

export const worker = new Worker("b-queue", async (job) => job.data);
