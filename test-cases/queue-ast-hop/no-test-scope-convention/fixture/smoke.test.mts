import { smokeQueue } from "./queues.mts";

export async function unmatchedWithoutConfiguredTestScope() {
  await smokeQueue.add("unhandled-smoke", {});
}
