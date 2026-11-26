import { FlowProducer, Queue, Worker } from "bullmq";
import { enqueue } from "./factory";

const queue = new Queue("dispatch");
const flow = new FlowProducer();
new Worker("dispatch", async () => {});
const label = "outer";

export function nested() {
  const label = "nested";
  queue.add(label, {}, { jobId: "nested:1" });
  queue.addBulk([{ name: label, data: {}, opts: { jobId: "nested:2" } }]);
  queue.upsertJobScheduler("timed", {}, { name: label, opts: { jobId: "nested:3" } });
  flow.add({ queueName: "dispatch", name: label, opts: { jobId: "nested:4" } });
  enqueue(queue, label, { jobId: "nested:5" });
}

export function masked(label: string) {
  queue.add(label, {}, { jobId: "masked:1" });
  queue.addBulk([{ name: label, data: {}, opts: { jobId: "masked:2" } }]);
  queue.upsertJobScheduler("masked", {}, { name: label, opts: { jobId: "masked:3" } });
  flow.add({ queueName: "dispatch", name: label, opts: { jobId: "masked:4" } });
  enqueue(queue, label, { jobId: "masked:5" });
}

export function forward() {
  queue.addBulk([{ name: lateLabel, data: {}, opts: { jobId: lateId } }]);
}

queue.addBulk([{ name: label, data: {}, opts: { jobId: "outer:1" } }]);
const lateLabel = "forward";
const lateId = "forward:1";
