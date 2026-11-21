import { FlowProducer } from "bullmq";

const flow = new FlowProducer();

export function enqueuePlainRoute(id: string) {
  return flow.add({ name: "planRoute", queueName: "routes", data: { id } });
}

export function enqueueRushRoute(id: string) {
  return flow.add({
    name: "rushRoute",
    queueName: "routes",
    data: { id },
    opts: { priority: 6 },
  });
}

export function enqueueFastRoute(id: string) {
  return flow.add({
    name: "fastRoute",
    queueName: "routes",
    data: { id },
    opts: { priority: 1 },
  });
}
