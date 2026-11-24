import { Queue, Worker } from "bullmq";

const widgets = new Queue("widgets", { defaultJobOptions: { removeOnComplete: true } });
new Worker("widgets", async () => {});

export async function enqueue(id: string) {
  await widgets.add("build", { id });
}

export async function recover(id: string) {
  const job = await widgets.getJob(id);
  if (await job?.isFailed()) await job.retry();
}
