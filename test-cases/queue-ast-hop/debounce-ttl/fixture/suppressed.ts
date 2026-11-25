import { eventsQueue } from "./queues";

export async function enqueueSuppressed() {
  // no-mistakes-disable-next-line queues-check: intentional legacy config
  await eventsQueue.add("suppressed", {}, { deduplication: { id: "s1", mode: "debounce", ttl: 1000 } });
  await eventsQueue.add("suppressed", {}, { deduplication: { id: "s2", mode: "debounce", ttl: 1000 } }); // no-mistakes-disable-line queues-check
}
