import { FlowProducer, Queue } from "glide-mq";

const queue = new Queue("events");
const absent = undefined;
const ttl = 1000;
const jobs = [
  { name: "bulk-const", data: {}, opts: { deduplication: { id: "a", mode: "debounce", ttl: 1000 } } },
];
const cleanJobs = [
  { name: "bulk-clean", data: {}, opts: { deduplication: { id: "b", mode: "debounce" } } },
];
declare const dynamicJobs: Array<{ name: string; data: object; opts: object }>;
declare const overrides: { deduplication?: { ttl?: number } };
declare const enabled: boolean;
declare const ttlSource: { ttl: number };
declare function getQueueOptions(): object;
declare function readTtl(): number;
declare function selectKey(): string;

function scopeMarker(): never {
  throw new Error("scope");
}
void scopeMarker;

export async function cases(): Promise<void> {
  await queue.add("alias-absent", {}, {
    deduplication: { id: "c", mode: "debounce", ttl: absent },
  });
  await queue.add("alias-number", {}, {
    deduplication: { id: "d", mode: "debounce", ttl },
  });
  await queue.add("later-spread", {}, {
    deduplication: { id: "e", mode: "debounce", ttl: 1000 },
    ...overrides,
  });
  await queue.add("spread-then-key", {}, {
    ...overrides,
    deduplication: { id: "f", mode: "debounce", ttl: 1000 },
  });
  await queue.addBulk(jobs);
  await queue.addBulk(cleanJobs);
  await queue.addBulk(dynamicJobs);
  const dynamicQueue = new Queue("dynamic", getQueueOptions());
  const plainQueue = new Queue("plain", {});
  void dynamicQueue;
  void plainQueue;
  const flow = new FlowProducer();
  await flow.add({
    name: "parent",
    queueName: "events",
    data: {},
    opts: {},
    ...overrides,
    children: [
      { name: "visible-child", queueName: "events", data: {}, opts: { deduplication: { id: "g", mode: "debounce", ttl: 50 } } },
      { name: "flow-as", queueName: "events", data: {}, opts: { deduplication: { id: "fa", mode: "debounce", ttl: 9 } } } as { name: string },
    ],
  });
  const undefined = 1000;
  await queue.add("shadow-ttl", {}, { deduplication: { id: "sh", mode: "debounce", ttl: undefined } });
  await queue.add("void-ttl", {}, { deduplication: { id: "vo", mode: "debounce", ttl: void 0 } });
  await queue.add("cond-ttl", {}, { deduplication: { id: "cd", mode: "debounce", ttl: enabled ? 1000 : undefined } });
  await queue.add("member-ttl", {}, { deduplication: { id: "mb", mode: "debounce", ttl: ttlSource.ttl } });
  await queue.add("call-ttl", {}, { deduplication: { id: "cl", mode: "debounce", ttl: readTtl() } });
  const satisfied = { deduplication: { id: "sf", mode: "debounce", ttl: 5 } } satisfies Record<string, unknown>;
  await queue.add("satisfies-opts", {}, satisfied);
  const asserted = { deduplication: { id: "az", mode: "debounce", ttl: 6 } } as Record<string, unknown>;
  await queue.add("assert-opts", {}, asserted);
  await queue.add("computed-key", {}, { deduplication: { id: "ck", mode: "debounce", ttl: 7 }, [selectKey()]: { id: "hidden" } });
  await queue.add("computed-then-explicit", {}, { [selectKey()]: { id: "hidden" }, deduplication: { id: "ce", mode: "debounce", ttl: 8 } });
  await queue.addBulk([{ name: "bulk-as", data: {}, opts: { deduplication: { id: "ba", mode: "debounce", ttl: 4 } } } as { name: string }]);
  await queue.addBulk([ // bulk-spread-rest
    ...dynamicJobs,
    { name: "bulk-rest", data: {}, opts: { deduplication: { id: "br", mode: "debounce", ttl: 3 } } },
  ]);
}
