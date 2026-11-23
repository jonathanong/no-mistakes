import { Queue } from "bullmq";
import * as bullmq from "bullmq";
import DefaultLimiter, { cycleA, cyclicExport, renamedWorker } from "./reexport";
import * as reexportNs from "./reexport";
import defaultHelper, { bullNsExport, renamedUnknown, unknownExport } from "./source";
import { missingWorker } from "./missing";

declare const globalWorker: { rateLimit(ms: number): Promise<void> };
declare const getHolder: () => {
  nested: { rateLimit(ms: number): Promise<void> };
  Ctor: new () => { rateLimit(ms: number): Promise<void> };
};
declare const getCtor: () => new () => { rateLimit(ms: number): Promise<void> };

class LocalClass {
  static async rateLimit(_ms: number) {}
}

function getDynamic(): { rateLimit(ms: number): Promise<void> } {
  return { async rateLimit(_ms: number) {} };
}

export async function runEdges() {
  let uninitialized: { rateLimit(ms: number): Promise<void> } | undefined;
  void uninitialized;

  const fromImport = renamedWorker;
  await fromImport.rateLimit(100);

  const fromNs = reexportNs.renamedWorker;
  await fromNs.rateLimit(200);

  const bullAlias = bullmq;
  await (bullAlias as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(300);
  await (bullNsExport as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(310);

  const classAlias = LocalClass;
  await classAlias.rateLimit(400);

  await (Queue as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(500);
  await (bullmq.Queue as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(550);
  await (defaultHelper as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(600);
  await DefaultLimiter.rateLimit(700);
  await new (getCtor())().rateLimit(710);
  await new (getHolder().Ctor)().rateLimit(720);

  await getDynamic().rateLimit(800);
  await globalWorker.rateLimit(900);
  await unknownExport.rateLimit(1000);
  await renamedUnknown.rateLimit(1010);
  await cycleA.rateLimit(1100);
  await (cyclicExport as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(1110);
  await missingWorker.rateLimit(1200);
  await getHolder().nested.rateLimit(1210);
  const localCycleA: { rateLimit(ms: number): Promise<void> } = localCycleB;
  const localCycleB: { rateLimit(ms: number): Promise<void> } = localCycleA;
  await localCycleA.rateLimit(1220);
  await (123 as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(1300);
  await (({ rateLimit: async () => {} })).rateLimit(1400);

  function shadowedNs(reexportNs: { renamedWorker: { rateLimit(ms: number): Promise<void> } }) {
    return reexportNs.renamedWorker.rateLimit(1500);
  }
  await shadowedNs({ renamedWorker: getDynamic() });
}
