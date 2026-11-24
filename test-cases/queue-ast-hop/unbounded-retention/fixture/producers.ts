import { FlowProducer } from "bullmq";
import { bare, bounded, dynamicQueue, unboundedDefault } from "./queues";

declare const dynamicValue: boolean | number;
declare const extraOpts: Record<string, unknown>;
declare const unknownReceiver: { add: (n: string, d: object, o?: object) => void };

const keepForever = false;
const keepBounded = { count: 250 };

// Accepted via queue defaults.
bounded.add("ok-default", {});

// Accepted via explicit call options (number, object with count, object with age, const).
bare.add("ok-number", {}, { removeOnComplete: 50, removeOnFail: 100 });
bare.add("ok-age", {}, { removeOnComplete: { age: 3600 }, removeOnFail: keepBounded });

// Reported: producer overrides bounded default with removeOnFail: false.
bounded.add("override-false", {}, { removeOnFail: false });

// Reported: const binding resolving to false.
bounded.add("const-false", {}, { removeOnComplete: keepForever });

// Reported: nothing bounds removeOnFail (only removeOnComplete is set).
// Must NOT also emit missing-job-option for removeOnFail.
bare.add("missing-fail", {}, { removeOnComplete: { count: 100 } });

// Reported: neither key is set, and attempts is also missing (dedup: missing-job-option only lists attempts).
bare.add("missing-both", {});

// Accepted: non-literal value abstains for removeOnFail.
bare.add("dynamic-abstain", {}, { removeOnComplete: true, removeOnFail: dynamicValue });

// Accepted: open spread abstains.
bare.add("spread-abstain", {}, { ...extraOpts });

// Accepted: dynamic default abstains.
dynamicQueue.add("dynamic-default", {});

// Reported once at defaultJobOptions, not duplicated per producer when inherited.
unboundedDefault.add("inherits-unbounded-default", {});

// Reported in addBulk and upsertJobScheduler.
bare.addBulk([
  { name: "bulk-ok", data: {}, opts: { removeOnComplete: true, removeOnFail: 10 } },
  { name: "bulk-false", data: {}, opts: { removeOnComplete: true, removeOnFail: false } },
]);

bare.upsertJobScheduler(
  "sched-unbounded",
  { every: 60_000 },
  { name: "sweep", data: {}, opts: { removeOnComplete: true, removeOnFail: false } },
);

// FlowProducer root and child.
const flow = new FlowProducer();
flow.add({
  name: "flow-root",
  queueName: "bounded",
  data: {},
  children: [{ name: "flow-child", queueName: "bare", data: {}, opts: { removeOnComplete: true } }],
});

// Unresolved receiver abstains.
unknownReceiver.add("ghost", {}, { removeOnFail: false });

// Suppressed via queues-check directive.
// no-mistakes-disable-next-line queues-check
bare.add("suppressed-job", {}, { removeOnComplete: true });

// Accepted: spread inside retention object resolves bounded const or abstains on unknown spread.
bare.add(
  "ok-keep-spread",
  {},
  { removeOnComplete: { ...keepBounded }, removeOnFail: { ...extraOpts } },
);
