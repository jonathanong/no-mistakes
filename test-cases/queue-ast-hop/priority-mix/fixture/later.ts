import { later } from "./queues";

export function enqueueHeld(id: string) {
  return later.add("holdLater", { id }, { priority: RUSH });
}

const RUSH = 4;

export function enqueueOpen(id: string) {
  return later.add("openLater", { id });
}
