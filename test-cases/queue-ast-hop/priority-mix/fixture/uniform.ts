import { holds, items, orders } from "./queues";

const NORMAL = 1;
const ZERO = (0);
const NONE = ZERO;

export function enqueueStandardItem(id: string) {
  return items.add("storeItem", { id }, { priority: NORMAL });
}

export function enqueueUrgentItem(id: string) {
  return items.add("storeItem", { id }, { priority: 3 });
}

export function enqueueOpenOrder(id: string) {
  return orders.add("placeOrder", { id });
}

export function enqueueHeldOrder(id: string) {
  return orders.add("holdOrder", { id }, { priority: 0 });
}

export function enqueueDraftOrder(id: string) {
  return orders.add("draftOrder", { id }, { priority: NONE });
}

export function enqueueFreeHold(id: string) {
  return holds.add("freeHold", { id }, { priority: NONE });
}

export function enqueuePaidHold(id: string) {
  return holds.add("paidHold", { id }, { priority: 3 });
}

export function enqueueFirmHold(id: string) {
  return holds.add("firmHold", { id }, { priority: 3 });
}
