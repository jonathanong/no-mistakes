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

const childAlias1 = flowChild;
const childAlias2 = childAlias1;
const childAlias3 = childAlias2;
const childAlias4 = childAlias3;
const childAlias5 = childAlias4;
const childAlias6 = childAlias5;
const childAlias7 = childAlias6;
const childAlias8 = childAlias7;
const childAlias9 = childAlias8;
const childAlias10 = childAlias9;
const childNotObject = [flowChild];
const cycDedup: object = { ...cycDedup };

const bulkBracketJobs = [
  { name: "bulk-bracket", data: {}, opts: { deduplication: { id: "bb", mode: "debounce", ttl: 12 } } },
];
bulkBracketJobs["push"]({ name: "later", data: {}, opts: {} });
const bulkOptionalJobs = [
  { name: "bulk-optional", data: {}, opts: { deduplication: { id: "bo", mode: "debounce", ttl: 15 } } },
];
bulkOptionalJobs?.push({ name: "later", data: {}, opts: {} });
declare const bulkDeclared: Array<{ name: string; data: object; opts: object }>;

function scopeMarker(): never {
  throw new Error("scope");
}
void scopeMarker;

const spreadChain0 = { mode: "debounce", ttl: 1 };
const spreadChain1 = { ...spreadChain0 };
const spreadChain2 = { ...spreadChain1 };
const spreadChain3 = { ...spreadChain2 };
const spreadChain4 = { ...spreadChain3 };
const spreadChain5 = { ...spreadChain4 };
const spreadChain6 = { ...spreadChain5 };
const spreadChain7 = { ...spreadChain6 };
const spreadChain8 = { ...spreadChain7 };
const spreadChain9 = { ...spreadChain8 };
const spreadChain10 = { ...spreadChain9 };
const spreadChain11 = { ...spreadChain10 };
const spreadChain12 = { ...spreadChain11 };
const spreadChain13 = { ...spreadChain12 };
const spreadChain14 = { ...spreadChain13 };
const spreadChain15 = { ...spreadChain14 };
const spreadChain16 = { ...spreadChain15 };
const spreadChain17 = { ...spreadChain16 };
const spreadChain18 = { ...spreadChain17 };
const spreadChain19 = { ...spreadChain18 };
const spreadChain20 = { ...spreadChain19 };
const spreadChain21 = { ...spreadChain20 };
const spreadChain22 = { ...spreadChain21 };
const spreadChain23 = { ...spreadChain22 };
const spreadChain24 = { ...spreadChain23 };
const spreadChain25 = { ...spreadChain24 };
const spreadChain26 = { ...spreadChain25 };
const spreadChain27 = { ...spreadChain26 };
const spreadChain28 = { ...spreadChain27 };
const spreadChain29 = { ...spreadChain28 };
const spreadChain30 = { ...spreadChain29 };
const spreadChain31 = { ...spreadChain30 };
const spreadChain32 = { ...spreadChain31 };
const spreadChain33 = { ...spreadChain32 };
const spreadChain34 = { ...spreadChain33 };
const spreadChain35 = { ...spreadChain34 };

const dedupChain0 = { id: "u14", mode: "debounce", ttl: 1 };
const dedupChain1 = dedupChain0;
const dedupChain2 = dedupChain1;
const dedupChain3 = dedupChain2;
const dedupChain4 = dedupChain3;
const dedupChain5 = dedupChain4;
const dedupChain6 = dedupChain5;
const dedupChain7 = dedupChain6;
const dedupChain8 = dedupChain7;
const dedupChain9 = dedupChain8;
const dedupChain10 = dedupChain9;
const dedupChain11 = dedupChain10;
const dedupChain12 = dedupChain11;
const dedupChain13 = dedupChain12;
const dedupChain14 = dedupChain13;
const dedupChain15 = dedupChain14;
const dedupChain16 = dedupChain15;
const dedupChain17 = dedupChain16;
const dedupChain18 = dedupChain17;
const dedupChain19 = dedupChain18;
const dedupChain20 = dedupChain19;
const dedupChain21 = dedupChain20;
const dedupChain22 = dedupChain21;
const dedupChain23 = dedupChain22;
const dedupChain24 = dedupChain23;
const dedupChain25 = dedupChain24;
const dedupChain26 = dedupChain25;
const dedupChain27 = dedupChain26;
const dedupChain28 = dedupChain27;
const dedupChain29 = dedupChain28;
const dedupChain30 = dedupChain29;
const dedupChain31 = dedupChain30;
const dedupChain32 = dedupChain31;
const dedupChain33 = dedupChain32;
const dedupChain34 = dedupChain33;
const dedupChain35 = dedupChain34;
const dedupChain36 = dedupChain35;
const dedupChain37 = dedupChain36;
const dedupChain38 = dedupChain37;
const dedupChain39 = dedupChain38;
const dedupChain40 = dedupChain39;
const dedupChain41 = dedupChain40;
const dedupChain42 = dedupChain41;
const dedupChain43 = dedupChain42;
const dedupChain44 = dedupChain43;
const dedupChain45 = dedupChain44;
const dedupChain46 = dedupChain45;
const dedupChain47 = dedupChain46;
const dedupChain48 = dedupChain47;
const dedupChain49 = dedupChain48;
const dedupChain50 = dedupChain49;
const dedupChain51 = dedupChain50;
const dedupChain52 = dedupChain51;
const dedupChain53 = dedupChain52;
const dedupChain54 = dedupChain53;
const dedupChain55 = dedupChain54;
const dedupChain56 = dedupChain55;
const dedupChain57 = dedupChain56;
const dedupChain58 = dedupChain57;
const dedupChain59 = dedupChain58;
const dedupChain60 = dedupChain59;
const dedupChain61 = dedupChain60;
const dedupChain62 = dedupChain61;
const dedupChain63 = dedupChain62;
const dedupChain64 = dedupChain63;
const dedupChain65 = dedupChain64;
const dedupChain66 = dedupChain65;
const dedupChain67 = dedupChain66;
const dedupChain68 = dedupChain67;
const dedupChain69 = dedupChain68;

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
  await flow.add({ name: "flow-alias-deep", queueName: "events", data: {}, opts: {}, children: [childAlias10, childNotObject, unboundChild] });
  await queue.add("dedup-cycle", {}, { deduplication: cycDedup });
  await queue.add("spread-budget", {}, { deduplication: { id: "u11", ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ ...{ mode: "debounce", ttl: 1 } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } } });
  await queue.add("spread-chain-budget", {}, { deduplication: { id: "u13", ...spreadChain35 } });
  await queue.add("dedup-chain-budget", {}, { deduplication: dedupChain69 });
  await flow.add({ name: "flow-undeclared", queueName: "events", data: {}, opts: {}, ...undeclaredFlow, children: [] });
  await queue.addBulk(bulkBracketJobs);
  await queue.addBulk(bulkOptionalJobs);
  await queue.addBulk(bulkDeclared);
  await queue.addBulk(lateJobs);
  await queue.add("ttl-late", {}, { deduplication: { id: "u12", mode: "debounce", ttl: lateTtl } });
}

const lateJobs = [
  { name: "bulk-late", data: {}, opts: { deduplication: { id: "bt", mode: "debounce", ttl: 16 } } },
];
const lateTtl = 5;
