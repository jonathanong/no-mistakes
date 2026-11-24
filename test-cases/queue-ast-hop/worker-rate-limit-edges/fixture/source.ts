import { Worker } from "bullmq";
import * as bullmq from "bullmq";
export { cyclicExport } from "./reexport";

export class BaseEdgeWorker extends Worker {}
export class PlainEdgeBase {}

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
