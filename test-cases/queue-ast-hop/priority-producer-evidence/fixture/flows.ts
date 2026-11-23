import { FlowProducer } from "bullmq";

const flow = new FlowProducer();
flow.add({ name: "flowRush", queueName: "crossFileFlow", opts: { priority: 2 } });
flow.add({ name: "flowRush", queueName: "ambiguousFlow", opts: { priority: 2 } });
