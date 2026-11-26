import { Worker } from "bullmq";
import { MAIL_QUEUE } from "@acme/queues-mail/config";

export const mailWorker = new Worker(MAIL_QUEUE, async (job) => {
  if (job.name === "welcome") {
    return;
  }
});
