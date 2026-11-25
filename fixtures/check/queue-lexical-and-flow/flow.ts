import { FlowProducer } from "bullmq";

const flow = new FlowProducer();

export function addCycle() {
  flow.add({ ...paint, opts: { priority: 1 } });
}

const paint = {
  queueName: "items",
  children: [{ ...draw, name: "draw-child", opts: { priority: 1 } }],
};
const draw = {
  queueName: "items",
  children: [{ ...paint, name: "paint-child", opts: {} }],
};

const left = { ...right, queueName: "items", name: "left" };
const right = { ...left };
flow.add({ ...left, ...right, opts: {} });
