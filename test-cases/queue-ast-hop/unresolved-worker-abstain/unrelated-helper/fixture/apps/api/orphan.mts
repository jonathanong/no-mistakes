import { Queue } from "bullmq";

const orphanQueue = new Queue("orphan");

export async function enqueueOrphan() {
  await orphanQueue.add("ping", { id: "o1" });
}
