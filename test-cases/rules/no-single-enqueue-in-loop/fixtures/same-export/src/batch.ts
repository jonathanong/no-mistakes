import { enqueueItems } from "./widgets";

export function publish(batches: readonly (readonly string[])[]): void {
  for (const batch of batches) {
    enqueueItems(batch); // silent:for-of
  }
}
