import { Queue, Worker } from "bullmq";
import * as queues from "./queues.mts";
import { ALERTS_QUEUE, alertsQueue } from "./queues.mts";

const BASE_QUEUE = "alerts";
const ALIAS_BASE = BASE_QUEUE;
const IMPORTED_ALIAS = ALERTS_QUEUE;

export async function sameFileWorkersMatchWithoutTestScope(unknownObj: { other: string }) {
  const rawQueue = new Queue(ALIAS_BASE);
  const rawWorker = new Worker(rawQueue.name, async () => {});
  await rawQueue.add("matched-by-queue-name", {});

  const importedNameQueue = new Queue(IMPORTED_ALIAS);
  const importedNameWorker = new Worker(importedNameQueue.name, async () => {});
  await importedNameQueue.add("matched-by-imported-alias", {});

  const importedQueueWorker = new Worker(alertsQueue.name, async () => {});

  const nsQueue = queues.createQueue(queues.ALERTS_QUEUE);
  const nsWorker = queues.createWorker(queues.ALERTS_QUEUE, async () => {});
  await nsQueue.add("matched-by-ns-factory", {});

  const unknownWorker = new Worker(unknownObj.other, async (job) => {
    if (job.name === "unrelated") {
      return;
    }
  });

  return {
    rawWorker,
    importedNameWorker,
    importedQueueWorker,
    nsWorker,
    unknownWorker,
  };
}
