import { Queue, Worker } from "bullmq";
import * as queues from "./queues.mts";

export async function sameFileWorkersMatchWithoutTestScope() {
  const localAlias = queues.ALERTS_QUEUE;
  const rawQueue = new Queue(localAlias);
  const rawWorker = new Worker(rawQueue.name, async () => {});
  await rawQueue.add("matched-by-queue-name", {});

  const nsQueue = queues.createQueue(queues.ALERTS_QUEUE);
  const nsWorker = queues.createWorker(queues.ALERTS_QUEUE, async () => {});
  await nsQueue.add("matched-by-ns-factory", {});

  return { rawWorker, nsWorker };
}
