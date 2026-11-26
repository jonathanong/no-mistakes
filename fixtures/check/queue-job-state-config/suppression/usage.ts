import { Queue, Worker } from "bullmq";

const widgets = new Queue("widgets");
new Worker("widgets", async () => {});

export async function retryFailed(id: string) {
  const job = await widgets.getJob(id);
  if ((await job?.getState()) === "failed") await job.retry();
}

export async function retryInline(id: string) {
  const job = await widgets.getJob(id);
  if ((await job?.getState()) === "failed") await job.retry(); // no-mistakes-disable-line queues-check: fixture exception
}
