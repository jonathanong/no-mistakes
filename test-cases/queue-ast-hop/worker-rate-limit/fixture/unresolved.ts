import { Worker } from "bullmq";
import { getUnknownWorker } from "./factory";

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
}

export async function throttleMember(holder: {
  worker: { rateLimit(ms: number): Promise<void> };
}) {
  await holder.worker.rateLimit(1000);
}
