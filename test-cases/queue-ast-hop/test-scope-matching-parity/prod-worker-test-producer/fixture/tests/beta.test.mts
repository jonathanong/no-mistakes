import { betaQueue } from "../src/queues.mts";

export async function enqueueBetaInTest() {
  await betaQueue.add("sync", { id: "b1" });
}
