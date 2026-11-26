// no-mistakes-disable-file no-single-enqueue-in-loop
import { enqueueItem } from "./widgets";

export function publish(items: string[]): void {
  for (const item of items) {
    enqueueItem(item);
  }
}
