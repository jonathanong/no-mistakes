import { Queue, Worker } from "bullmq";
import { ordersQueue } from "../src/queue";

export async function testOrderFlow() {
  const localQueue = new Queue("orders-test-isolated");
  const localWorker = new Worker("orders-test-isolated", async (job) => {
    if (job.name === "create") {
      return job.id;
    }
    return null;
  });
  void localWorker;
  await localQueue.add("create", { id: "t1" });

  const job = await ordersQueue.getJob("t1");
  if ((await job?.getState()) === "failed") {
    await job.retry();
  }
}
