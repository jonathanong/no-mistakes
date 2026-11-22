import { drafts, holds, items, orders, widgets } from "./queues";

export function enqueueWidgets() {
  return Promise.all([widgets.add("pack", {}), widgets.add("ship", {})]);
}

export function enqueueItems() {
  return Promise.all([items.add("pack", {}), items.add("ship", {})]);
}

export function enqueueOrders() {
  return orders.add("pack", {});
}

export function enqueueDrafts(name: string) {
  return drafts.add(name, {});
}

export function enqueueHolds() {
  return Promise.all([holds.add("pack", {}), holds.add("ship", {})]);
}
