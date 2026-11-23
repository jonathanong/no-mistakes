import { TestWorker, Worker } from "glide-mq";

export async function startDynamic(queueName: string) {
  const dynamicWorker = new Worker(queueName, async () => {});
  await dynamicWorker.rateLimit(1000);

  const testWorker = new TestWorker("reports", async () => {});
  await testWorker.rateLimit(500);
}
