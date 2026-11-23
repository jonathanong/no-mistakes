import { FlowProducer } from "bullmq";

declare const unresolved: object;
const toExempt = { queueName: "orders" };
const toRequired = { queueName: "items" };
const target = { ...toRequired };

const flow = new FlowProducer();

flow.add({ name: "flow-to-exempt", ...{ queueName: "items" }, ...toExempt, data: {}, opts: {} });
flow.add({ name: "flow-to-required", ...{ queueName: "orders" }, ...target, data: {}, opts: {} });
flow.add({ name: "flow-unknown-after", ...{ queueName: "items" }, ...unresolved, data: {}, opts: {} });
flow.add({ ...unresolved, queueName: "items", name: "flow-explicit-after", data: {}, opts: {} });
