import type { Worker as QueueWorker } from "bullmq";
import { createWorker } from "./worker-factory";

export const worker = createWorker("items", async (_task, handle: QueueWorker) => {
  await handle.rateLimit(1000);
  await (async (handle: QueueWorker) => handle.rateLimit(1000))(handle);
});
