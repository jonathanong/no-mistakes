import { FlowProducer } from "bullmq";

const paint = {
  queueName: "items",
  name: "cycle-paint",
  children: [{ ...draw, name: "draw-child", opts: { priority: 1 } }],
};
const draw = {
  queueName: "items",
  name: "cycle-draw",
  children: [{ ...paint, name: "paint-child", opts: { priority: 1 } }],
};

const flow = new FlowProducer();
flow.add({ ...paint, opts: { priority: 1 } });
