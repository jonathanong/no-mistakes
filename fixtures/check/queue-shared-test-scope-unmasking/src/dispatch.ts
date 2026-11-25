import { Queue, Worker } from "bullmq";

export const dispatchQueue = new Queue("dispatch");

export const dispatchWorker = new Worker("dispatch", async (job) => {
  if (job.name === "send-email" || job.name === "send-sms") {
    throw Worker.RateLimitError();
  }
});

export async function enqueueDispatch(id: string) {
  await dispatchQueue.add("send-email", { id });
  await dispatchQueue.add("send-sms", { id });
}
