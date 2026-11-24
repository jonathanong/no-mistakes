import { Queue } from "bullmq";
import { orphanQueue } from "@acme/queues-mail/config";

export async function testProducerWithoutWorker() {
  await orphanQueue.add("test-only-job", { id: "t1" });
  const isolated = new Queue("test-isolated-queue");
  await isolated.add("isolated-job", {});
}
