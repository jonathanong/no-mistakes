import { BaseEdgeWorker, renamedWorker } from "./source";
import * as sourceNs from "./source";
export { renamedWorker };
export { cyclicExport } from "./source";
export { Queue as ReexportedQueue, Worker as ReexportedBullWorker } from "bullmq";
export * from "bullmq";
export * from "./star-empty";
export * from "./source";
export const nsWorkerExport = sourceNs.renamedWorker;
const localNsWorker = sourceNs.renamedWorker;
export { localNsWorker as reexportedNsWorker };
export class CrossFileSubWorker extends BaseEdgeWorker {}
export class NsSubWorker extends sourceNs.BaseEdgeWorker {}
class RenamedCrossSub extends BaseEdgeWorker {}
export { RenamedCrossSub as AliasedCrossSub };
class RenamedNsSub extends sourceNs.BaseEdgeWorker {}
export { RenamedNsSub as AliasedNsSub };
const localConstructed = new BaseEdgeWorker();
export { localConstructed as aliasedConstructed };
const localNsConstructed = new sourceNs.BaseEdgeWorker();
export { localNsConstructed as aliasedNsConstructed };
const localImpAlias = renamedWorker;
export { localImpAlias as aliasedImpWorker };
export const crossFileSubInstance = new BaseEdgeWorker();
export const nsSubInstance = new sourceNs.BaseEdgeWorker();
export const subViaLocal = new CrossFileSubWorker();
export const subViaNsLocal = new NsSubWorker();
const localSub = CrossFileSubWorker;
void localSub;
export const cycleA = cycleB;
export const cycleB = cycleA;
export default class DefaultLimiter {
  static async rateLimit(_ms: number) {}
}
