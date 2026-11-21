import { enqueueItem, enqueueItems } from "./widgets";
import { enqueueItem as queueWidget } from "./widgets";
import * as widgets from "./widgets";
import { enqueueItem as enqueueOrder } from "./orders";

export async function publish(
  items: readonly string[],
  batches: readonly (readonly string[])[],
  order: string,
  limit: number,
): Promise<void> {
  for (const item of items) {
    enqueueItem(item); // report:for-of
  }
  for (let index = 0; index < items.length; index += 1) {
    enqueueItem(items[index]); // report:for
  }
  for (const key in items) {
    enqueueItem(items[key]); // report:for-in
  }
  let index = 0;
  while (index < items.length) {
    enqueueItem(items[index]); // report:while
    index += 1;
  }
  let cursor = 0;
  do {
    enqueueItem(items[cursor]); // report:do
    cursor += 1;
  } while (cursor < items.length);
  items.map(enqueueItem); // report:map callback
  items.map((item) => enqueueItem(item)); // report:map callback
  items.forEach((item) => {
    enqueueItem(item); // report:forEach callback
  });
  items.flatMap(enqueueItem); // report:flatMap callback
  for (const item of items) {
    queueWidget(item); // report:for-of
  }
  for (const item of items) {
    widgets.enqueueItem(item); // report:for-of
  }
  for (let index = 0; index < limit; index += 1) {
    enqueueItem(order); // report:for
  }
  for (const item of items) {
    void item;
    enqueueItem(order); // report:for-of
  }
  const copied = [order];
  for (const item of copied) {
    enqueueItem(item); // report:for-of
  }
  await enqueueItems(items); // silent:bulk-once
  for (const batch of batches) {
    enqueueItems(batch); // silent:bulk-loop
  }
  enqueueItem(order); // silent:outside
  for (const item of items) {
    enqueueOrder(item); // silent:orders
  }
  for (let index = 0; index < 1; index += 1) {
    enqueueItem(order); // silent:once
  }
  const closed = ["widget"] as const;
  for (const item of closed) {
    enqueueItem(item); // silent:closed
  }
  for (const item of items) {
    queueOne(item); // silent:wrapper
  }
  for (const item of items) {
    const later = () => enqueueItem(item); // silent:nested
    later();
  }
  items.filter((item) => {
    enqueueItem(item); // silent:filter
    return true;
  });
  for (const item of items) {
    // no-mistakes-disable-next-line no-single-enqueue-in-loop -- one reviewed widget send
    enqueueItem(item); // silent:suppressed
  }
}

function queueOne(item: string): void {
  enqueueItem(item); // silent:helper
}
