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
declare const queueOptions: object[];
declare const mysteryChild: { name: string };
const d0 = { mode: "debounce", ttl: 1000 };
const d1 = { ...d0, ...d0 };
const d2 = { ...d1, ...d1 };
const d3 = { ...d2, ...d2 };
const d4 = { ...d3, ...d3 };
const d5 = { ...d4, ...d4 };
const d6 = { ...d5, ...d5 };
const mutableJobs = [
  { name: "bulk-push", data: {}, opts: { deduplication: { id: "p", mode: "debounce", ttl: 9 } } },
];
mutableJobs.push({ name: "later-job", data: {}, opts: {} });
const flowChild = { name: "flow-child", queueName: "events", data: {}, opts: { deduplication: { id: "fc", mode: "debounce", ttl: 100 } } };

declare function readDedup(): object;
declare function makeDedup(): object;
declare function readChildren(): object[];
declare const ghostSource: object;
let reassignedTtl = 5;
const nullDedup = null;
const opaqueDedup = readDedup();
const flowShared = { name: "shared-flow", queueName: "events", data: {}, opts: {} };
const flowOpaque = readChildren();
const flowLoopA: object = { name: "loop-a", ...flowShared };
let bulkLetJobs = [
  { name: "bulk-let", data: {}, opts: { deduplication: { id: "bl", mode: "debounce", ttl: 11 } } },
];
const aliasA = 1;
const aliasB = aliasA;
const aliasC = aliasB;
const aliasD = aliasC;
const aliasE = aliasD;
const aliasF = aliasE;
const aliasG = aliasF;
const aliasH = aliasG;
const aliasI = aliasH;
const aliasJ = aliasI;
declare const ttlNoInit: number;

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
  await queue.add("null-spread", {}, {
    deduplication: { id: "ns", mode: "debounce", ttl: 1000, ...null },
  });
  await queue.add("spread-bomb", {}, { deduplication: { id: "sb", ...d6 } });
  await queue.addBulk(mutableJobs);
  const spreadQueue = new Queue("spread-queue", ...queueOptions);
  void spreadQueue;
  await flow.add({
    name: "parent-id",
    queueName: "events",
    data: {},
    opts: {},
    children: [flowChild, mysteryChild],
  });
  await queue.add("spread-unbound", {}, { deduplication: { id: "u1", mode: "debounce", ttl: 1, ...unboundDedup } });
  await queue.add("spread-null-const", {}, { deduplication: { id: "u2", mode: "debounce", ttl: 2, ...nullDedup } });
  await queue.add("spread-opaque-const", {}, { deduplication: { id: "u3", ...opaqueDedup } });
  await queue.add("spread-call", {}, { deduplication: { id: "u4", ...makeDedup() } });
  await queue.add("computed-dedup-key", {}, { deduplication: { id: "u5", [selectKey()]: 1 } });
  await queue.add("dedup-unbound", {}, { deduplication: unboundDedup });
  await queue.add("dedup-call", {}, { deduplication: makeDedup() });
  await queue.add("mode-null", {}, { deduplication: { id: "u6", mode: null, ttl: 3 } });
  await queue.add("ttl-unbound", {}, { deduplication: { id: "u7", mode: "debounce", ttl: unboundTtl } });
  await queue.add("ttl-reassigned", {}, { deduplication: { id: "u8", mode: "debounce", ttl: reassignedTtl } });
  await queue.add("ttl-no-init", {}, { deduplication: { id: "u9", mode: "debounce", ttl: ttlNoInit } });
  await queue.add("ttl-deep-alias", {}, { deduplication: { id: "u10", mode: "debounce", ttl: aliasJ } });
  await queue.addBulk([, { name: "bulk-elision", data: {}, opts: { deduplication: { id: "be", mode: "debounce", ttl: 13 } } }]);
  await queue.addBulk(bulkLetJobs);
  await queue.addBulk([[[[[[[[[[{ name: "bulk-deep", data: {}, opts: { deduplication: { id: "bd", mode: "debounce", ttl: 14 } } }]]]]]]]]]]);
  await flow.add({ name: "flow-ghost", queueName: "events", data: {}, opts: {}, ...ghostSource, children: [] });
  await flow.add({ name: "flow-opaque", queueName: "events", data: {}, opts: {}, ...flowOpaque, children: [] });
  await flow.add({ name: "flow-loop", queueName: "events", data: {}, opts: {}, ...flowLoopA, ...flowLoopA, children: [] });
  await flow.add({ name: "flow-computed", queueName: "events", data: {}, opts: {}, [selectKey()]: 1, children: [] });
}
