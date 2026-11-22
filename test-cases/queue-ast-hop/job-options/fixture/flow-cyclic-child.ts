import { FlowProducer } from "bullmq";

const cycle = {
  queueName: "items",
  name: "cyclic-parent",
  children: [{ ...cycle, name: "cyclic-child", opts: { priority: 1 } }],
};

const flow = new FlowProducer();
flow.add({ ...cycle, opts: { priority: 1 } });
