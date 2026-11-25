import { eventsQueue } from "@acme/queues-events/config";

export async function publishEvent() {
  await eventsQueue.add("published", { id: "e1" });
}
