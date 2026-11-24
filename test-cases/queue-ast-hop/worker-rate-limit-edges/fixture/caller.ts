import { Queue, Worker } from "bullmq";
import * as bullmq from "bullmq";
import DefaultLimiter, {
  AliasedCrossSub,
  AliasedNsSub,
  CrossFileSubWorker,
  NsSubWorker,
  ReexportedBullWorker,
  ReexportedQueue,
  aliasedConstructed,
  aliasedImpWorker,
  aliasedNsConstructed,
  crossFileSubInstance,
  cycleA,
  cyclicExport,
  nsSubInstance,
  nsWorkerExport,
  reexportedNsWorker,
  renamedWorker,
  starOnlyWorker,
  subViaLocal,
  subViaNsLocal,
} from "./reexport";
import * as reexportNs from "./reexport";
import defaultHelper, {
  BaseEdgeWorker,
  DerivedMissingBase,
  NullBaseExport,
  PlainEdgeBase,
  bullNsExport,
  createEdgeWorker,
  externalMissingInst,
  helperCtorInst,
  renamedUnknown,
  unknownExport,
} from "./source";
import * as sourceNs from "./source";
import { SubOfNamedDefaultWorker } from "./default-class";
import DefaultAliasedWorker from "./default-alias";
import DefaultCrossAliasedWorker from "./default-cross-alias";
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
  class LocalExtQueueId extends Queue {}
  class LocalExtDynCall extends getCtor() {}
  class LocalExtDynMember extends getHolder().Ctor {}
  const anonClass = class {};
  void anonClass;
  const subFromNs = new sourceNs.BaseEdgeWorker();
  const nsCallResult = sourceNs.default() as unknown as { rateLimit(ms: number): Promise<void> };
  const dynNewLocal = new (getCtor())();
  const queueCtorLocal = bullmq.Queue;
  void queueCtorLocal;
  const holderObj = { x: 0 };
  holderObj.x = 1;
  await new LocalExtImported().rateLimit(1600);
  await new LocalExtNs().rateLimit(1610);
  await subFromNs.rateLimit(1620);
  await new sourceNs.BaseEdgeWorker().rateLimit(1625);
  await new CrossFileSubWorker().rateLimit(1630);
  await new NsSubWorker().rateLimit(1640);
  await new AliasedCrossSub().rateLimit(1645);
  await crossFileSubInstance.rateLimit(1650);
  await nsSubInstance.rateLimit(1660);
  await subViaLocal.rateLimit(1662);
  await subViaNsLocal.rateLimit(1664);
  await new LocalExtPlain().rateLimit(1670);
  await (new LocalExtQueue("q") as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(
    1680,
  );
  await (new LocalExtQueueId("q") as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(
    1682,
  );
  await (ReexportedQueue as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(1684);
  await dynNewLocal.rateLimit(1686);
  await new LocalExtDynCall().rateLimit(1687);
  await new LocalExtDynMember().rateLimit(1688);
  await (defaultHelper() as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(1689);
  await (sourceNs.default() as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(1690);
  await nsCallResult.rateLimit(1692);
  let mutatedImported = unknownExport;
  mutatedImported = getDynamic();
  await mutatedImported.rateLimit(1700);
  let mutatedNs = reexportNs.renamedWorker;
  mutatedNs = getDynamic();
  await mutatedNs.rateLimit(1710);
  let mutatedAlias = fromImport;
  mutatedAlias = getDynamic();
  await mutatedAlias.rateLimit(1720);

  let compoundAcc: any = { async rateLimit(_ms: number) {} };
  compoundAcc += new Worker("edge-jobs", async () => {}) as any;
  await compoundAcc.rateLimit(1730);
  let logicalWorker: { rateLimit(ms: number): Promise<void> } | undefined;
  logicalWorker ??= new Worker("edge-jobs", async () => {});
  await logicalWorker.rateLimit(1740);

  class LocalNullBase extends null {
    async rateLimit(_ms: number) {}
  }
  await new LocalNullBase().rateLimit(1750);
  await new NullBaseExport().rateLimit(1760);
  await helperCtorInst.rateLimit(1770);

  const localSubCtor = LocalExtImported;
  await new localSubCtor().rateLimit(1780);
  const directWorkerCtor = Worker;
  await new directWorkerCtor("edge-jobs", async () => {}).rateLimit(1790);
  await (directWorkerCtor as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(1795);
  const impCtorAlias = BaseEdgeWorker;
  const impCtorInst = new impCtorAlias();
  await impCtorInst.rateLimit(1800);
  const nsCtorAlias = sourceNs.BaseEdgeWorker;
  await new nsCtorAlias().rateLimit(1810);
  const plainCtorAlias = PlainEdgeBase;
  await (new plainCtorAlias() as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(
    1815,
  );

  await new AliasedNsSub().rateLimit(1820);
  await aliasedConstructed.rateLimit(1830);
  await aliasedNsConstructed.rateLimit(1840);
  await aliasedImpWorker.rateLimit(1850);

  await new SubOfNamedDefaultWorker().rateLimit(1860);
  await new DefaultAliasedWorker().rateLimit(1870);
  await new DefaultCrossAliasedWorker().rateLimit(1880);

  let mutPlain: { rateLimit(ms: number): Promise<void> };
  mutPlain = new PlainEdgeBase() as unknown as { rateLimit(ms: number): Promise<void> };
  await mutPlain.rateLimit(1890);
  let mutWorkerSub: { rateLimit(ms: number): Promise<void> };
  mutWorkerSub = new BaseEdgeWorker();
  await mutWorkerSub.rateLimit(1900);

  class SubOfReexportedBullWorker extends ReexportedBullWorker {}
  await new SubOfReexportedBullWorker("edge-jobs", async () => {}).rateLimit(1910);
  await (ReexportedBullWorker as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(
    1915,
  );

  await new DerivedMissingBase().rateLimit(1920);
  await externalMissingInst.rateLimit(1925);
}
