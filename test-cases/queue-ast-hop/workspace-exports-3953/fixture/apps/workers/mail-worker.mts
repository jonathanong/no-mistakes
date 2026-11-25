import { Worker } from "bullmq";
import { BILLING_QUEUE, QUEUE_NAME } from "@example/queues-mail/config";

export const mailWorker = new Worker(QUEUE_NAME, async (job) => {
  if (job.name === "welcome") {
    return;
  }
});

export const billingWorker = new Worker(BILLING_QUEUE, async (job) => {
  if (job.name === "charge") {
    return;
  }
});
