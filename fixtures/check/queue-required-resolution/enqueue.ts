import { FlowProducer } from "bullmq";
import { primary } from "./queue-primary";
import { secondary } from "./queue-secondary";

declare const tools: { tags: typeof primary };
declare function getTools(): typeof tools;

tools.tags.addBulk([{ name: "tag-bulk", data: {} }]);
getTools().tags.upsertJobScheduler("tag-schedule", {}, { name: "tag-scheduler" });

primary.addBulk([{ name: "real-bulk", data: {} }]);
secondary.upsertJobScheduler("real-schedule", {}, { name: "real-scheduler" });

const flow = new FlowProducer();
flow.add({ queueName: "shared", name: "ambiguous-flow", data: {} });
flow.add({ queueName: "shared", name: "explicit-flow", data: {}, opts: { attempts: 1, backoff: 1 } });
flow.add({ queueName: "local", name: "ambiguous-local-flow", data: {} });
