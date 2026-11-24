import { orphanQueue } from "@acme/queues-mail/config";

export async function enqueueOrphan() {
  await orphanQueue.add("never-handled", { id: "o1" });
}
