import { Queue, Worker } from "bullmq";
import * as bullmq from "bullmq";
export { cyclicExport } from "./reexport";

declare const getBase: () => new () => { rateLimit(ms: number): Promise<void> };

export class BaseEdgeWorker extends Worker {}
export class PlainEdgeBase {}
export class QueueSub extends Queue {}
export class BullQueueSub extends bullmq.Queue {}
export class DynamicModSub extends getBase() {}
export const modQueueInst = new Queue("edge-jobs");
export const modBullQueueInst = new bullmq.Queue("edge-jobs");
export const modBullQueueCtor = bullmq.Queue;
export const modDynNew = new (getBase())();

export function createEdgeWorker(name: string, processor: () => Promise<void>) {
  return new Worker(name, processor);
}

const innerWorker = new Worker("edge-jobs", async () => {});
const innerAlias = innerWorker;
export { innerAlias as renamedWorker };
export const starOnlyWorker = new Worker("edge-jobs", async () => {});
export const unknownExport = getDynamic();
export { unknownExport as renamedUnknown };
export const bullNsExport = bullmq;
let topLevelLet = 0;
const { ignored } = { ignored: topLevelLet };
void ignored;

function getDynamic(): { rateLimit(ms: number): Promise<void> } {
  return { async rateLimit(_ms: number) {} };
}

export default function defaultHelper() {}
