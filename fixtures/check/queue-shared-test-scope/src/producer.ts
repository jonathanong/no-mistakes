import { ordersQueue } from "./queue";

export async function enqueueOrder(id: string) {
  await ordersQueue.add("create", { id });
}

export async function retryFromQueue(id: string) {
  const job = await ordersQueue.getJob(id);
  if ((await job?.getState()) === "failed") {
    await job.retry();
  }
}
