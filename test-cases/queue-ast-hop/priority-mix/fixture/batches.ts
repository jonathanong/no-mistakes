import { batches } from "./queues";

const RUSH = +2;

export function enqueuePlainBatch(id: string) {
  return batches.add("countBatch", { id });
}

export function enqueueRushBatch(id: string) {
  return batches.add("countBatch", { id }, { priority: RUSH });
}
