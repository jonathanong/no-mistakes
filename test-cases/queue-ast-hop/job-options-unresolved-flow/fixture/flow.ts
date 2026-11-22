import { FlowProducer } from "bullmq";

declare const unresolved: object;
const flow = new FlowProducer();

flow.add({
  name: "opaque-parent",
  opts: { priority: 1 },
  ...unresolved,
  queueName: "items",
  children: [{ queueName: "items", name: "opaque-child", opts: {} }],
});

flow.add({
  name: "known-parent",
  opts: { priority: 1 },
  queueName: "items",
  children: [{ queueName: "items", name: "known-child", opts: {} }],
});
