import { alphaQueue } from "./queues.mts";

export async function enqueueAlpha() {
  await alphaQueue.add("run", { id: "a1" });
}
