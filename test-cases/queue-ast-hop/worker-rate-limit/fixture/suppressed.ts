import { Worker } from "bullmq";

export const noticeWorker = new Worker("notices", async () => {});

export async function throttleSuppressed() {
  // no-mistakes-disable-next-line queues-check
  await noticeWorker.rateLimit(1000);
}
