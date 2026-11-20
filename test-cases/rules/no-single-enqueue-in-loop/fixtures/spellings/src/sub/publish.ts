import { enqueueItem, enqueueItems } from "../widgets";

export function publish(
  items: readonly string[],
  batches: readonly (readonly string[])[],
): void {
  for (const item of items) {
    enqueueItem(item); // report:for-of
  }
  for (const batch of batches) {
    enqueueItems(batch); // silent:bulk-loop
  }
}
