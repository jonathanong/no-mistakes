import { FlowProducer, Queue, Worker } from "bullmq";
import * as callbacks from "./enqueue";
import { enqueueConfigured, makeReviewQueue as buildQueue } from "./factory";

const direct = new Queue("review-direct");
direct.add("overwritten", {}, { jobId: "old:id", ["jobId"]: "safe-id" });
direct.add("effective", {}, { jobId: "safe-id", ["jobId"]: "new:id" });
callbacks.widgets.add("namespace", {}, { jobId: "namespace:id" });
(callbacks.widgets as Queue).add("wrapped", {}, { jobId: "wrapped:id" });

export function shadowNamespace(callbacks: { widgets: Queue }) {
  callbacks.widgets.add("shadow-namespace", {}, { jobId: "shadow:id" });
}

const factoryQueue = buildQueue("review-direct");
factoryQueue.add("factory-queue", {}, { jobId: "factory:id" });

export function shadowFactory(buildQueue: (name: string) => Queue) {
  const local = buildQueue("review-direct");
  local.add("shadow-factory", {}, { jobId: "shadow:id" });
}

export function shadowConstructor(Queue: new (name: string) => Queue) {
  const local = new Queue("review-direct");
  local.add("shadow-constructor", {}, { jobId: "shadow:id" });
}

export function shadowEnqueue(enqueueConfigured: (queue: Queue, job: string, options: object) => void) {
  enqueueConfigured(direct, "shadow-enqueue", { jobId: "shadow:id" });
}

export function shadowFlow(FlowProducer: new () => import("bullmq").FlowProducer) {
  const flow = new FlowProducer();
  flow.add({ name: "shadow-flow", queueName: "review-direct", opts: { jobId: "shadow:id" } });
}

declare const opaqueOptions: object;
declare const opaqueKey: string;
direct.add("spread-after", {}, { jobId: "old:id", ...opaqueOptions });
direct.add("computed-after", {}, { jobId: "old:id", [opaqueKey]: "maybe" });
direct.add("after-spread", {}, { ...opaqueOptions, jobId: "new:id" });
direct.add("after-computed", {}, { [opaqueKey]: "maybe", ["jobId"]: "new:id" });

new Worker("review-direct", async () => {});
