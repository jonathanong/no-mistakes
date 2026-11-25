import { Queue, Worker } from "bullmq";
import { LOCAL_QUEUE, MAIL_QUEUE, createQueue, createWorker } from "@acme/queues-mail/config";

export async function runLocalPair(prefix: string) {
  const rawQueue = new Queue(LOCAL_QUEUE, { prefix });
  const rawWorker = new Worker(LOCAL_QUEUE, async () => {}, { prefix });
  await rawQueue.add("ping-raw", {});

  const factoryQueue = createQueue(MAIL_QUEUE, { prefix });
  const factoryWorker = createWorker(MAIL_QUEUE, async () => {}, { prefix });
  await factoryQueue.add("ping-factory", {});

  return { rawWorker, factoryWorker };
}
