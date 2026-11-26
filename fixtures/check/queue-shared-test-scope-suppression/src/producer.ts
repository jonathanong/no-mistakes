import { ordersQueue } from "./queue";

export async function enqueueOrder(id: string) {
  await ordersQueue.add("create", { id });
}

export async function enqueueInline(id: string) {
  await ordersQueue.add("create", { id }); // no-mistakes-disable-line queues-check: fixture exception
}
