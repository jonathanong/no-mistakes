// no-mistakes-disable-file queues-check: fixture exception
import { ordersQueue } from "./queue";

export async function enqueueOrder(id: string) {
  await ordersQueue.add("create", { id });
}
