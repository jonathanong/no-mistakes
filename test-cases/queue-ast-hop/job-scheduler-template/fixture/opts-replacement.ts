import { Queue } from "bullmq";

const orders = new Queue("orders");
const early = { opts: { delay: 1 } };
const replacement = { opts: {} };
const tracking = { name: "tracked" };
const withId = { opts: { jobId: "scheduled-id" } };
declare const externalTemplate: object;
declare const externalOptions: object;

orders.upsertJobScheduler("spread-wins", {}, { opts: { delay: 2 }, ...replacement });
orders.upsertJobScheduler("explicit-wins", {}, { ...early, opts: {} });
orders.upsertJobScheduler("computed-wins", {}, { ...early, ["opts"]: {} });
orders.upsertJobScheduler("no-opts-in-spread", {}, { opts: { deduplication: { id: "x" } }, ...tracking });
orders.upsertJobScheduler("last-spread", {}, { ...early, ...withId });
orders.upsertJobScheduler("opaque-opts-replaced", {}, { opts: externalOptions, ...replacement });
orders.upsertJobScheduler("opaque-template", {}, { ...externalTemplate, opts: { delay: 3 } });
