import { Queue, Worker } from "bullmq";

export const triageQueue = new Queue("triage");

export const triageWorker = new Worker("triage", async (job) => {
  if (job.name === "normal" || job.name === "urgent") {
    return job.id;
  }
});

export async function enqueueTriage(id: string) {
  await triageQueue.add("normal", { id });
  await triageQueue.add("urgent", { id }, { priority: 2 });
}
