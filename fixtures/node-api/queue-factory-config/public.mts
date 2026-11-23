import { analyzeProject, queueCheck, queueEdges } from "no-mistakes";
import type {
  QueueFactories, QueueFactoriesOptions, QueueFactoryArgument,
  QueueEnqueueFactoryOptions, QueueCheckFinding, QueueEdge,
} from "no-mistakes";

const selector: QueueFactoryArgument = { arg: 0, property: "name" };
const legacy: QueueFactories = ["createQueue"];
const fixed: QueueEnqueueFactoryOptions = {
  callee: "makeFixedEnqueue", queueName: "background", job: selector,
  optionsOnReturnedCall: 1,
};
const roles: QueueFactoriesOptions = {
  queue: [{ callee: "createQueue", name: 0 }],
  worker: [{ callee: "createWorker", name: 0, processor: 1 }],
  enqueue: [
    fixed,
    { callee: "makeObjectEnqueue", queue: { arg: 0, property: "queueName" },
      job: { arg: 0, property: "jobName" }, defaults: { arg: 0, property: "defaults" },
      optionsOnReturnedCall: 1 },
    { callee: "makeReturnedEnqueue", queue: 0, job: 1, optionsOnReturnedCall: 1 },
  ],
};
// @ts-expect-error A selector and a fixed name are mutually exclusive.
const invalid: QueueEnqueueFactoryOptions = { callee: "makeFixedEnqueue", queue: 0, queueName: "background", job: 0 };
const checks: Promise<QueueCheckFinding[]> = queueCheck({ root: "." });
const edges: Promise<QueueEdge[]> = queueEdges({ root: "." });
void [selector, legacy, roles, invalid, checks, edges,
  analyzeProject({ root: ".", reports: [{ type: "queues" }, { type: "queueEdges" }, { type: "queueCheck" }] })];
