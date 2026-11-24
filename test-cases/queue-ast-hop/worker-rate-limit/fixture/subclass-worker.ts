import { Worker } from "bullmq";
import * as bullmq from "bullmq";
import { ExportedWorker, SubExportedWorker, exportedSubInstance } from "./subclass-export";

class ExampleWorker extends Worker {}

class ChildWorker extends ExampleWorker {}

class NsWorker extends bullmq.Worker {}

class PlainLimiter {
  async rateLimit(_ms: number) {}
}

class SubPlainLimiter extends PlainLimiter {}

export async function throttleSubclasses() {
  await new ExampleWorker().rateLimit(1000);
  const child = new ChildWorker();
  await child.rateLimit(900);
  await new NsWorker().rateLimit(800);
  await new ExportedWorker().rateLimit(700);
  await new SubExportedWorker().rateLimit(600);
  await exportedSubInstance.rateLimit(500);
  await (ExampleWorker as unknown as { rateLimit(ms: number): Promise<void> }).rateLimit(400);
  await new PlainLimiter().rateLimit(300);
  await new SubPlainLimiter().rateLimit(200);
}
