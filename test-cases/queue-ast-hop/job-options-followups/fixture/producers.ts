import { FlowProducer, Queue } from "bullmq";
import { sharedA } from "./queue-a";
import { sharedB } from "./queue-b";

declare function getThing(): {
  addBulk: (items: Array<{ name: string; data: object; opts?: object }>) => void;
  upsertJobScheduler: (id: string, repeat: object, template: object) => void;
};
declare const holder: {
  client: {
    addBulk: (items: Array<{ name: string; data: object; opts?: object }>) => void;
    upsertJobScheduler: (id: string, repeat: object, template: object) => void;
  };
};
declare const unknownClient: {
  addBulk: (items: Array<{ name: string; data: object; opts?: object }>) => void;
  upsertJobScheduler: (id: string, repeat: object, template: object) => void;
};

// Unresolved receivers must abstain for addBulk and upsertJobScheduler.
getThing().addBulk([{ name: "untracked-bulk", data: {} }]);
holder.client.addBulk([{ name: "member-bulk", data: {} }]);
unknownClient.addBulk([{ name: "unknown-bulk", data: {} }]);
getThing().upsertJobScheduler("sched-a", { every: 1000 }, { name: "untracked-sched", data: {} });
holder.client.upsertJobScheduler("sched-b", { every: 1000 }, { name: "member-sched", data: {} });
unknownClient.upsertJobScheduler("sched-c", { every: 1000 }, { name: "unknown-sched", data: {} });

// Imported bindings resolve to their own construction site's defaults.
sharedA.add("from-a", {});
sharedB.add("from-b", {});

// FlowProducer referencing a queue constructed in multiple files must abstain when defaults matter.
const flow = new FlowProducer();
flow.add({ name: "ambiguous-flow", queueName: "shared", data: {} });
flow.add({
  name: "covered-flow",
  queueName: "shared",
  data: {},
  opts: { attempts: 2, backoff: 500 },
});

// Same-file duplicate queue names use each local binding's own defaults.
const localFirst = new Queue("local-dup", { defaultJobOptions: { attempts: 2 } });
const localSecond = new Queue("local-dup", { defaultJobOptions: { backoff: 500 } });
localFirst.add("local-one", {});
localSecond.add("local-two", {});
