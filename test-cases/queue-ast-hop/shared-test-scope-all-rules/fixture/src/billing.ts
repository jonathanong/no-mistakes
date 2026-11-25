import { Queue, Worker } from "bullmq";

export const billingQueue = new Queue("billing", {
  defaultJobOptions: {
    attempts: 3,
    removeOnComplete: true,
    removeOnFail: { count: 100 },
  },
});

export const billingWorker = new Worker("billing", async (job) => {
  if (job.name === "charge") {
    await billingWorker.rateLimit(500);
  }
});

export async function chargeBilling(id: string) {
  await billingQueue.add("charge", { id });
}
