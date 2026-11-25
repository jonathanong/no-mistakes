import { Queue, Worker } from "bullmq";
import { MAIL_QUEUE, createQueue, createWorker } from "@acme/queues-mail/config";

export async function testRawAndFactory(prefix: string) {
  const rawQueue = new Queue(MAIL_QUEUE, { prefix });
  const rawWorker = new Worker(MAIL_QUEUE, async () => {}, { prefix });
  await rawQueue.add("test-raw-job", {});

  const factoryQueue = createQueue(MAIL_QUEUE, { prefix });
  const factoryWorker = createWorker(MAIL_QUEUE, async () => {}, { prefix });
  await factoryQueue.add("test-factory-job", {});

  return { rawWorker, factoryWorker };
}
