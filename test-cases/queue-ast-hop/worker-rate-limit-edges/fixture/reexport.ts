import { renamedWorker } from "./source";
export { renamedWorker };
export const cycleA = cycleB;
export const cycleB = cycleA;
export default class DefaultLimiter {
  static async rateLimit(_ms: number) {}
}
