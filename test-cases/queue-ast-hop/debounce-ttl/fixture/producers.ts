import { FlowProducer } from "glide-mq";
import { alertsQueue, eventsQueue, throttleQueue } from "./queues";

const DEBOUNCE_MODE = "debounce";
const DEBOUNCE_DEDUP = { id: "const-dedup", mode: "debounce", ttl: 5000 };
const DEBOUNCE_OPTS = { deduplication: { id: "const-opts", mode: "debounce", ttl: 5000 } };
const BASE_DEBOUNCE = { mode: "debounce" };

export async function enqueueCases(
  dynamicMode: "debounce" | "throttle",
  dynamicDedup: { id: string; mode?: string; ttl?: number },
  rest: Record<string, unknown>,
) {
  await eventsQueue.add("direct", {}, { deduplication: { id: "a", mode: "debounce", ttl: 1000 } });
  await eventsQueue.add("as-const", {}, { deduplication: { id: "b", mode: "debounce" as const, ttl: 1000 } });
  await eventsQueue.add("template", {}, { deduplication: { id: "c", mode: `debounce`, ttl: 1000 } });
  await eventsQueue.add("const-mode", {}, { deduplication: { id: "d", mode: DEBOUNCE_MODE, ttl: 1000 } });
  await eventsQueue.add("const-dedup", {}, { deduplication: DEBOUNCE_DEDUP });
  await eventsQueue.add("const-opts", {}, DEBOUNCE_OPTS);
  await eventsQueue.add("spread-const", {}, { deduplication: { id: "e", ...BASE_DEBOUNCE, ttl: 1000 } });

  await eventsQueue.addBulk([
    { name: "bulk-bad", data: {}, opts: { deduplication: { id: "f", mode: "debounce", ttl: 2000 } } },
    { name: "bulk-ok", data: {}, opts: { deduplication: { id: "g", mode: "throttle", ttl: 2000 } } },
  ]);

  const flow = new FlowProducer();
  await flow.add({
    name: "flow-root",
    queueName: "events",
    data: {},
    opts: { deduplication: { id: "h", mode: "debounce", ttl: 3000 } },
    children: [
      {
        name: "flow-child",
        queueName: "events",
        data: {},
        opts: { deduplication: { id: "i", mode: "debounce", ttl: 3000 } },
      },
    ],
  });

  // Clean cases
  await eventsQueue.add("clean-debounce", {}, { deduplication: { id: "j", mode: "debounce" } });
  await eventsQueue.add("clean-throttle", {}, { deduplication: { id: "k", mode: "throttle", ttl: 1000 } });
  await eventsQueue.add("clean-simple", {}, { deduplication: { id: "l" } });
  await eventsQueue.add("clean-undef-ttl", {}, { deduplication: { id: "m", mode: "debounce", ttl: undefined } });
  await alertsQueue.add("ping", {});
  await throttleQueue.add("ping", {});

  // Undecidable cases (must abstain with diagnostic, never silently pass)
  await eventsQueue.add("dyn-mode", {}, { deduplication: { id: "n", mode: dynamicMode, ttl: 1000 } });
  await eventsQueue.add("dyn-dedup", {}, { deduplication: dynamicDedup });
  await eventsQueue.add("dyn-spread", {}, { deduplication: { id: "o", mode: "debounce", ...rest } });
}
