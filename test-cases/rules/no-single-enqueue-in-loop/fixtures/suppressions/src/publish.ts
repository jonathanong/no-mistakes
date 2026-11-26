import { enqueueItem } from "./widgets";

export function publish(items: string[]): void {
  for (const item of items) {
    enqueueItem(item); // no-mistakes-disable-line no-single-enqueue-in-loop
  }
  for (const item of items) {
    enqueueItem(item);
  }
}
