import { labels } from "./queues";

export function enqueuePlainLabel(id: string) {
  // no-mistakes-disable-next-line queues-check
  return labels.add("printLabel", { id });
}

export function enqueueRushLabel(id: string) {
  return labels.add("rushLabel", { id }, { priority: 4 });
}
