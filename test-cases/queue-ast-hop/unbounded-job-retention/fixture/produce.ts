import { FlowProducer } from "bullmq";
import { submitJob } from "./factory";
import { blank, bounded, dynamic, factory, partial, retained, shared } from "./queues";
import { shared as otherShared } from "./other";

declare function getOptions(): object;
declare function getRetention(): boolean;
declare function getCap(): number;

blank.add("missing-both", {});
partial.add("missing-fail", {});
bounded.add("inherits-bounds", {});
retained.add("inherits-explicit-false", {});
factory.add("inherits-factory-false", {});
dynamic.add("inherits-unknown", {});
shared.add("bounded-same-name", {});
otherShared.add("unbounded-same-name", {});

bounded.add("literal-override", {}, { removeOnComplete: false });
bounded.add("closed-unbounded-object", {}, { removeOnFail: {} });
bounded.add("bounded-overrides", {}, { removeOnComplete: 0, removeOnFail: { age: 500 } });
bounded.add("dynamic-value", {}, { removeOnComplete: getRetention() });
bounded.add("dynamic-bound", {}, { removeOnComplete: { age: getCap() } });
bounded.add("dynamic-object", {}, getOptions());
bounded.add("open-last", {}, { removeOnComplete: false, ...getOptions() });
bounded.add("false-last", {}, { ...getOptions(), removeOnComplete: false });

const keep = { removeOnFail: false };
const expire = { removeOnFail: { count: 100 } };
bounded.add("bounded-last", {}, { ...keep, ...expire });
bounded.add("unbounded-last", {}, { ...expire, ...keep });
bounded.add("literal-last", {}, { ...keep, removeOnFail: true });
bounded.add("duplicate-keys", {}, { removeOnFail: false, removeOnFail: true });
bounded.addBulk([
  { name: "bulk-retained", data: {}, opts: { removeOnComplete: false } },
  { name: "bulk-expires", data: {}, opts: { removeOnComplete: true } },
]);
bounded.upsertJobScheduler("seen", { every: 60000 }, {
  name: "scheduler-retained", data: {}, opts: { removeOnFail: false },
});

const flow = new FlowProducer();
flow.add({
  name: "flow-literal", queueName: "bounded", data: {}, opts: { removeOnComplete: false },
  children: [{ name: "flow-missing", queueName: "blank", data: {} }],
});

submitJob(bounded, "configured-literal", {}, { removeOnFail: false });

// no-mistakes-disable-next-line queues-check
bounded.add("suppressed", {}, { removeOnFail: false });

declare const schedulerArgs: [string, object];
bounded.upsertJobScheduler(...schedulerArgs, {}, { opts: { removeOnFail: false } });

bounded.add("unbounded-retention-spread", {}, { removeOnFail: { ...{ limit: 10 } } });
bounded.add("bounded-retention-spread", {}, { removeOnFail: { ...{ count: 10 } } });
bounded.add("opaque-retention-spread", {}, { removeOnFail: { ...getOptions() } });
declare const boundName: string;
bounded.add("computed-retention-bound", {}, { removeOnFail: { [boundName]: 10 } });
bounded.add("retention-getter", {}, {
  get removeOnFail() { return getRetention(); },
});
