import { FlowProducer } from "bullmq";

const shared = {
  children: [{ queueName: "items", name: "shared-child", opts: {} }],
};

const flow = new FlowProducer();
flow.add({
  queueName: "items",
  name: "branch-parent",
  opts: { priority: 1 },
  children: [
    { ...shared, queueName: "items", name: "first-branch", opts: { priority: 1 } },
    { ...shared, queueName: "items", name: "second-branch", opts: { priority: 1 } },
  ],
});
