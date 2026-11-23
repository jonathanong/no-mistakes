import { Queue } from "bullmq";
import * as bullmq from "bullmq";
import DefaultLimiter, { cycleA, renamedWorker } from "./reexport";
import * as reexportNs from "./reexport";
import defaultHelper, { unknownExport } from "./source";
import { missingWorker } from "./missing";

declare const globalWorker: { rateLimit(ms: number): Promise<void> };

class LocalClass {
  static async rateLimit(_ms: number) {}
}

function getDynamic(): { rateLimit(ms: number): Promise<void> } {
  return { async rateLimit(_ms: number) {} };
}

export async function runEdges() {
  const fromImport = renamedWorker;
  await fromImport.rateLimit(100);

  const fromNs = reexportNs.renamedWorker;
  await fromNs.rateLimit(200);

  const bullAlias = bullmq;
  await (bullAlias as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(300);

  const classAlias = LocalClass;
  await classAlias.rateLimit(400);

  await (Queue as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(500);
  await (bullmq.Queue as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(550);
  await (defaultHelper as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(600);
  await DefaultLimiter.rateLimit(700);

  await getDynamic().rateLimit(800);
  await globalWorker.rateLimit(900);
  await unknownExport.rateLimit(1000);
  await cycleA.rateLimit(1100);
  await missingWorker.rateLimit(1200);
  await (123 as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(1300);
  await (({ rateLimit: async () => {} })).rateLimit(1400);
}
