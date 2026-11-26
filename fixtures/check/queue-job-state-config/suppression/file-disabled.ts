// no-mistakes-disable-file queues-check: fixture exception
import { Queue, Worker } from "bullmq";

const accounts = new Queue("accounts");
new Worker("accounts", async () => {});

export async function retryFailed(id: string) {
  const job = await accounts.getJob(id);
  if ((await job?.getState()) === "failed") await job.retry();
}
