import { BaseEdgeWorker, renamedWorker } from "./source";
import * as sourceNs from "./source";
export { renamedWorker };
export { cyclicExport } from "./source";
export * from "bullmq";
export * from "./source";
export const nsWorkerExport = sourceNs.renamedWorker;
const localNsWorker = sourceNs.renamedWorker;
export { localNsWorker as reexportedNsWorker };
export class CrossFileSubWorker extends BaseEdgeWorker {}
export class NsSubWorker extends sourceNs.BaseEdgeWorker {}
export const crossFileSubInstance = new BaseEdgeWorker();
export const nsSubInstance = new sourceNs.BaseEdgeWorker();
const localSub = CrossFileSubWorker;
void localSub;
export const cycleA = cycleB;
export const cycleB = cycleA;
export default class DefaultLimiter {
  static async rateLimit(_ms: number) {}
}
