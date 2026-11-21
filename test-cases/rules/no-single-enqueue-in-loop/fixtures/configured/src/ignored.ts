import { enqueueItem } from "./widgets";

export function publishIgnored(items: readonly string[]): void {
  for (const item of items) {
    enqueueItem(item); // silent:ignored
  }
}
