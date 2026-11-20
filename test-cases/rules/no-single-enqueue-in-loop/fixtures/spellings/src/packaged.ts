import { enqueueItem } from "../packages/widget-queue/index";

export function publish(items: readonly string[]): void {
  for (const item of items) {
    enqueueItem(item); // report:for-of target:widget-queue#enqueueItem
  }
}
