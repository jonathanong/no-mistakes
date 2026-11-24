import { Queue, Worker } from "bullmq";
import { ORPHAN_QUEUE, orphanQueue } from "@acme/queues-mail/config";

export async function testProducerWithoutWorker() {
  await orphanQueue.add("test-only-job", { id: "t1" });
  const isolated = new Queue("test-isolated-queue");
  await isolated.add("isolated-job", {});
  return new Worker(ORPHAN_QUEUE, async () => {});
}
