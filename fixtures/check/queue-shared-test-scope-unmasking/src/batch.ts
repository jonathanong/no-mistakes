import { Queue, Worker } from "bullmq";

export const batchQueue = new Queue("batch");

export const batchWorker = new Worker("batch", async (job) => {
  if (job.name === "standard" || job.name === "rush") {
    return job.id;
  }
});

export async function enqueueBatch(id: string) {
  await batchQueue.add("standard", { id });
  await batchQueue.add("rush", { id }, { priority: 1 });
}
