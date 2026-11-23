import { renamedWorker } from "./source";
export { renamedWorker };
export { cyclicExport } from "./source";
export const cycleA = cycleB;
export const cycleB = cycleA;
export default class DefaultLimiter {
  static async rateLimit(_ms: number) {}
}
