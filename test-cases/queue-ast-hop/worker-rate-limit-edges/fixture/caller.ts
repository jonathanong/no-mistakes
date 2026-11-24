import { Queue } from "bullmq";
import * as bullmq from "bullmq";
import DefaultLimiter, {
  CrossFileSubWorker,
  NsSubWorker,
  crossFileSubInstance,
  cycleA,
  cyclicExport,
  nsSubInstance,
  nsWorkerExport,
  reexportedNsWorker,
  renamedWorker,
  starOnlyWorker,
} from "./reexport";
import * as reexportNs from "./reexport";
import defaultHelper, {
  BaseEdgeWorker,
  PlainEdgeBase,
  bullNsExport,
  createEdgeWorker,
  renamedUnknown,
  unknownExport,
} from "./source";
import * as sourceNs from "./source";
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

  await nsWorkerExport.rateLimit(210);
  await reexportedNsWorker.rateLimit(220);
  await starOnlyWorker.rateLimit(230);
  await (await createEdgeWorker("edge-jobs", async () => {})).rateLimit(240);

  const bullAlias = bullmq;
  await (bullAlias as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(300);
  await (bullNsExport as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(310);

  const classAlias = LocalClass;
  await classAlias.rateLimit(400);

  await (Queue as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(500);
  await (bullmq.Queue as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(550);
  await (new bullmq.Queue("edge-jobs") as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(560);
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

  class LocalExtImported extends BaseEdgeWorker {}
  class LocalExtNs extends sourceNs.BaseEdgeWorker {}
  class LocalExtPlain extends PlainEdgeBase {}
  class LocalExtQueue extends bullmq.Queue {}
  const subFromNs = new sourceNs.BaseEdgeWorker();
  await new LocalExtImported().rateLimit(1600);
  await new LocalExtNs().rateLimit(1610);
  await subFromNs.rateLimit(1620);
  await new CrossFileSubWorker().rateLimit(1630);
  await new NsSubWorker().rateLimit(1640);
  await crossFileSubInstance.rateLimit(1650);
  await nsSubInstance.rateLimit(1660);
  await new LocalExtPlain().rateLimit(1670);
  await (new LocalExtQueue("q") as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(
    1680,
  );
  await (sourceNs.default() as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(1690);
  let mutatedImported = unknownExport;
  mutatedImported = getDynamic();
  await mutatedImported.rateLimit(1700);
  let mutatedNs = reexportNs.renamedWorker;
  mutatedNs = getDynamic();
  await mutatedNs.rateLimit(1710);
}
