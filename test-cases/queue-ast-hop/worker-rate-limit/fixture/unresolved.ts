import { Worker } from "bullmq";
import { getUnknownWorker } from "./factory";
import { apiLimiter } from "./skipped-limiter";
import { externalLimiter } from "external-limiter";

declare const dynamicBase: new () => { rateLimit(ms: number): Promise<void> };

class DynamicBaseWorker extends dynamicBase {}

export async function throttleParam(target: { rateLimit(ms: number): Promise<void> }) {
  await target.rateLimit(1000);
}

export async function throttleUnknownFactory() {
  const unknownWorker = getUnknownWorker();
  await unknownWorker.rateLimit(1000);
}

export async function throttleMutated() {
  let mutable = new Worker("notices", async () => {});
  mutable = getUnknownWorker() as unknown as typeof mutable;
  await mutable.rateLimit(1000);

  let assignedLater: { rateLimit(ms: number): Promise<void> } | undefined;
  assignedLater = new Worker("notices", async () => {});
  await assignedLater.rateLimit(1000);
}

export async function throttleMember(holder: {
  worker: { rateLimit(ms: number): Promise<void> };
}) {
  await holder.worker.rateLimit(1000);
}

export async function throttleUnrelatedAndDynamicBase() {
  await apiLimiter.rateLimit(1000);
  await externalLimiter.rateLimit(1000);
  await new DynamicBaseWorker().rateLimit(1000);
}
