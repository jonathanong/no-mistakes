import { gammaQueue } from "../src/queues.mts";

export async function enqueueDynamicInTest(jobName: string) {
  await gammaQueue.add(jobName, { id: "g1" });
}
