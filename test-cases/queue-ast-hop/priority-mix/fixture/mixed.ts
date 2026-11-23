import { widgets } from "./queues";

export function enqueuePlainWidget(id: string) {
  return widgets.add("packWidget", { id });
}

export function enqueueRushWidget(id: string) {
  return widgets.add("rushWidget", { id }, { priority: 5 });
}

export function enqueueFastWidget(id: string, extra: object) {
  return widgets.add("fastWidget", { id }, { ...extra, priority: 2 as const });
}
