import { enqueueItem } from "./widgets";

export function publish(items: readonly string[]): void {
  for (const item of items) {
    enqueueItem(item);
  }
}
